import pkg from '../package.json'
import { existsSync, readdirSync, readFileSync } from 'fs'
import { dirname, extname, join, resolve } from 'path'
import { ensureBootstrapMacro } from './bootstrapMacro'

ensureBootstrapMacro()

type MissingImport = {
  importer: string
  specifier: string
}

function scanFiles(dir: string, out: string[]): void {
  if (!existsSync(dir)) return
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      scanFiles(fullPath, out)
      continue
    }
    if (['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs'].includes(extname(entry.name))) {
      out.push(fullPath)
    }
  }
}

function hasResolvableTarget(basePath: string): boolean {
  const withoutJs = basePath.replace(/\.js$/u, '')
  const candidates = [
    withoutJs,
    `${withoutJs}.ts`,
    `${withoutJs}.tsx`,
    `${withoutJs}.js`,
    `${withoutJs}.jsx`,
    `${withoutJs}.mjs`,
    `${withoutJs}.cjs`,
    join(withoutJs, 'index.ts'),
    join(withoutJs, 'index.tsx'),
    join(withoutJs, 'index.js'),
  ]
  return candidates.some(candidate => existsSync(candidate))
}

function collectMissingRelativeImports(): MissingImport[] {
  const files: string[] = []
  scanFiles(resolve('src'), files)
  scanFiles(resolve('vendor'), files)
  const missing: MissingImport[] = []
  const seen = new Set<string>()
  const pattern =
    /(?:import|export)\s+[\s\S]*?from\s+['"](\.\.?\/[^'"]+)['"]|require\(\s*['"](\.\.?\/[^'"]+)['"]\s*\)/g

  for (const file of files) {
    const text = readFileSync(file, 'utf8')
    for (const match of text.matchAll(pattern)) {
      const specifier = match[1] ?? match[2]
      if (!specifier) continue
      const target = resolve(dirname(file), specifier)
      if (hasResolvableTarget(target)) continue
      const key = `${file} -> ${specifier}`
      if (seen.has(key)) continue
      seen.add(key)
      missing.push({
        importer: file,
        specifier,
      })
    }
  }

  return missing.sort((a, b) =>
    `${a.importer}:${a.specifier}`.localeCompare(`${b.importer}:${b.specifier}`),
  )
}

const args = process.argv.slice(2)
const missingImports = collectMissingRelativeImports()

if (args.includes('--version')) {
  if (missingImports.length > 0) {
    console.log(`${pkg.version} (restored dev workspace)`)
    console.log(`missing_relative_imports=${missingImports.length}`)
    process.exit(0)
  }
  console.log(pkg.version)
  process.exit(0)
}

if (args.includes('--help')) {
  if (missingImports.length > 0) {
    console.log('Claude Code restored development workspace')
    console.log(`version: ${pkg.version}`)
    console.log(`missing relative imports: ${missingImports.length}`)
    process.exit(0)
  }
  console.log('Usage: claude [options] [prompt]')
  console.log('')
  console.log('Basic restored commands:')
  console.log('  --help       Show this help')
  console.log('  --version    Show version')
  console.log('')
  console.log('Interactive REPL startup is routed to src/main.tsx when run without these flags.')
  process.exit(0)
}

if (missingImports.length > 0) {
  console.log('Claude Code restored development workspace')
  console.log(`version: ${pkg.version}`)
  console.log(`missing relative imports: ${missingImports.length}`)
  console.log('')
  console.log('Top missing modules:')
  for (const item of missingImports.slice(0, 20)) {
    console.log(`- ${item.importer.replace(`${process.cwd()}/`, '')} -> ${item.specifier}`)
  }
  console.log('')
  console.log('The original app entry is still blocked by missing restored sources.')
  console.log('Use this workspace to continue restoration; once missing imports reach 0, this launcher will forward to src/main.tsx automatically.')
  process.exit(0)
}

// Route through the original CLI bootstrap so the exported `main()` is
// actually invoked. Importing `main.tsx` directly only evaluates the module.
await import('./entrypoints/cli.tsx')
