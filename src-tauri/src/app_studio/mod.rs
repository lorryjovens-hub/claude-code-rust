use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/* ═══════════════════════════════════════════════
   App Studio Engine
   Generates React Native / Expo projects from
   design specifications, manages device previews,
   and orchestrates backend code generation.
   ═══════════════════════════════════════════════ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProjectSpec {
    pub name: String,
    pub template: String,        // expo-router, expo-tabs, expo-auth, expo-realtime
    pub screens: Vec<ScreenSpec>,
    pub api_endpoints: Vec<ApiEndpointSpec>,
    pub database_type: String,   // sqlite, postgres, none
    pub deployment: DeployConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSpec {
    pub name: String,
    pub route: String,
    pub components: Vec<String>,
    pub has_api: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointSpec {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub platform: String,        // expo, android, ios, harmony
    pub use_expo: bool,
    pub dev_server_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub files: Vec<GeneratedFile>,
    pub install_commands: Vec<String>,
    pub deploy_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

pub struct AppStudio {
    workspace_dir: PathBuf,
}

impl AppStudio {
    pub fn new(workspace_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&workspace_dir).ok();
        Self { workspace_dir }
    }

    /// Generate a complete Expo/React Native project from spec
    pub fn generate_project(&self, spec: &AppProjectSpec) -> Result<ProjectStructure, String> {
        let mut files = Vec::new();
        let project_dir = self.workspace_dir.join(&spec.name);
        std::fs::create_dir_all(&project_dir).ok();

        // package.json
        files.push(GeneratedFile {
            path: format!("{}/package.json", spec.name),
            content: self.generate_package_json(spec),
        });

        // App.tsx — entry point with navigation
        files.push(GeneratedFile {
            path: format!("{}/App.tsx", spec.name),
            content: self.generate_app_tsx(spec),
        });

        // Generate screen files
        for screen in &spec.screens {
            files.push(GeneratedFile {
                path: format!("{}/screens/{}.tsx", spec.name, screen.name),
                content: self.generate_screen(screen, spec),
            });
        }

        // API layer
        if !spec.api_endpoints.is_empty() {
            files.push(GeneratedFile {
                path: format!("{}/api/client.ts", spec.name),
                content: self.generate_api_client(spec),
            });
            // Generate backend server
            files.push(GeneratedFile {
                path: format!("{}/server/index.ts", spec.name),
                content: self.generate_server(spec),
            });
            files.push(GeneratedFile {
                path: format!("{}/server/package.json", spec.name),
                content: r#"{"name":"api-server","version":"1.0.0","scripts":{"dev":"tsx watch index.ts"},"dependencies":{"express":"^4.18","cors":"^2.8","sqlite3":"^5.1","jsonwebtoken":"^9.0"}}"#.to_string(),
            });
        }

        // tsconfig.json
        files.push(GeneratedFile {
            path: format!("{}/tsconfig.json", spec.name),
            content: r#"{"compilerOptions":{"strict":true,"target":"esnext","module":"commonjs","jsx":"react-native","moduleResolution":"node","allowSyntheticDefaultImports":true,"esModuleInterop":true},"exclude":["node_modules"]}"#.to_string(),
        });

        // Write files to disk
        for file in &files {
            let full_path = self.workspace_dir.join(&file.path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&full_path, &file.content).map_err(|e| format!("Write error: {}", e))?;
        }

        Ok(ProjectStructure {
            files,
            install_commands: vec!["npm install".to_string()],
            deploy_command: if spec.deployment.use_expo {
                "npx expo start".to_string()
            } else {
                "npx react-native start".to_string()
            },
        })
    }

    fn generate_package_json(&self, spec: &AppProjectSpec) -> String {
        let deps = match spec.template.as_str() {
            "expo-auth" => r#"{"expo":"~52.0","react":"18.3","react-native":"0.76","@react-navigation/native":"^7","@react-navigation/bottom-tabs":"^7","expo-secure-store":"~14","@react-native-async-storage/async-storage":"^2"}"#,
            "expo-realtime" => r#"{"expo":"~52.0","react":"18.3","react-native":"0.76","socket.io-client":"^4.7","@react-navigation/native":"^7","react-native-gifted-chat":"^2"}"#,
            _ => r#"{"expo":"~52.0","react":"18.3","react-native":"0.76","expo-router":"~4","@react-navigation/native":"^7","nativewind":"^4"}"#,
        };
        format!(r#"{{"name":"{}","version":"1.0.0","main":"expo-router/entry","scripts":{{"start":"expo start","android":"expo start --android","ios":"expo start --ios","web":"expo start --web"}},"dependencies":{}}}"#, spec.name, deps)
    }

    fn generate_app_tsx(&self, _spec: &AppProjectSpec) -> String {
        format!(r#"import React from 'react';
import Navigation from './screens/Navigation';

export default function App() {{
  return <Navigation />;
}}
"#)
    }

    fn generate_screen(&self, screen: &ScreenSpec, _spec: &AppProjectSpec) -> String {
        let api_imports = if screen.has_api {
            "import { api } from '../api/client';"
        } else { "" };

        format!(r#"import React, {{ useState, useEffect }} from 'react';
import {{ View, Text, StyleSheet, TouchableOpacity, FlatList }} from 'react-native';
{api_imports}

export default function {name}Screen() {{
  return (
    <View style={{styles.container}}>
      <Text style={{styles.title}}>{name}</Text>
      <Text style={{styles.subtitle}}>Generated by Claude App Studio</Text>
    </View>
  );
}}

const styles = StyleSheet.create({{
  container: {{ flex: 1, backgroundColor: '#f5f5f5', padding: 20 }},
  title: {{ fontSize: 24, fontWeight: '700', color: '#1a1a1a', marginBottom: 8 }},
  subtitle: {{ fontSize: 14, color: '#666', marginBottom: 24 }},
}});
"#, name = screen.name, api_imports = api_imports)
    }

    fn generate_api_client(&self, spec: &AppProjectSpec) -> String {
        let base_url = if spec.deployment.use_expo { "'http://localhost:3001'" } else { "'http://10.0.2.2:3001'" };
        format!(r#"const API_BASE = {base_url};

interface ApiOptions {{
  method?: string;
  body?: any;
  token?: string;
}}

export async function api(path: string, opts: ApiOptions = {{}}) {{
  const headers: Record<string, string> = {{ 'Content-Type': 'application/json' }};
  if (opts.token) headers['Authorization'] = `Bearer ${{opts.token}}`;
  const res = await fetch(`${{API_BASE}}${{path}}`, {{
    method: opts.method || 'GET',
    headers,
    body: opts.body ? JSON.stringify(opts.body) : undefined,
  }});
  if (!res.ok) throw new Error(`API ${{res.status}}`);
  return res.json();
}}

export const auth = {{
  login: (email: string, password: string) => api('/auth/login', {{ method: 'POST', body: {{ email, password }} }}),
  register: (data: any) => api('/auth/register', {{ method: 'POST', body: data }}),
  profile: (token: string) => api('/user/profile', {{ token }}),
}};
"#, base_url = base_url)
    }

    fn generate_server(&self, spec: &AppProjectSpec) -> String {
        format!(r#"import express from 'express';
import cors from 'cors';

const app = express();
app.use(cors());
app.use(express.json());

// Health check
app.get('/api/health', (_req, res) => {{
  res.json({{ status: 'ok', project: '{name}' }});
}});

// Auth endpoints
app.post('/auth/login', (req, res) => {{
  const {{ email, password }} = req.body;
  if (!email || !password) {{
    return res.status(400).json({{ error: 'Email and password required' }});
  }}
  res.json({{ token: 'demo-token', user: {{ email }} }});
}});

app.post('/auth/register', (req, res) => {{
  res.json({{ success: true }});
}});

app.get('/user/profile', (_req, res) => {{
  res.json({{ name: 'Demo User', avatar: null }});
}});

// Generated API endpoints
{api_routes}

const PORT = process.env.PORT || 3001;
app.listen(PORT, () => {{
  console.log(`API server running on port ${{PORT}}`);
}});
"#, name = spec.name, api_routes = self.generate_api_routes(spec))
    }

    fn generate_api_routes(&self, spec: &AppProjectSpec) -> String {
        spec.api_endpoints.iter().map(|ep| {
            format!(
                r#"app.{method}('{path}', (req, res) => {{
  res.json({{ message: '{handler} endpoint', data: null }});
}});
"#,
                method = ep.method.to_lowercase(),
                path = ep.path,
                handler = ep.handler
            )
        }).collect::<Vec<_>>().join("\n")
    }

    /// Generate a QR-code URL for Expo Go
    pub fn get_expo_url(&self, port: u16) -> String {
        format!("exp://127.0.0.1:{}", port)
    }
}
