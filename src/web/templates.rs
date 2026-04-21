//! HTML Templates - Template engine for web pages

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }

    /// Render the main index page
    pub fn render_index(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Claude Code Plugin Marketplace</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            color: #fff;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        header {{
            text-align: center;
            padding: 3rem 0;
        }}
        
        h1 {{
            font-size: 3rem;
            margin-bottom: 1rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }}
        
        .subtitle {{
            font-size: 1.2rem;
            color: #a0a0a0;
            margin-bottom: 2rem;
        }}
        
        .search-box {{
            max-width: 600px;
            margin: 0 auto 3rem;
            position: relative;
        }}
        
        .search-box input {{
            width: 100%;
            padding: 1rem 1.5rem;
            font-size: 1.1rem;
            border: 2px solid rgba(255,255,255,0.1);
            border-radius: 50px;
            background: rgba(255,255,255,0.05);
            color: #fff;
            outline: none;
            transition: all 0.3s;
        }}
        
        .search-box input:focus {{
            border-color: #667eea;
            background: rgba(255,255,255,0.1);
        }}
        
        .search-box input::placeholder {{
            color: #666;
        }}
        
        .categories {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 3rem;
        }}
        
        .category {{
            background: rgba(255,255,255,0.05);
            padding: 1.5rem;
            border-radius: 12px;
            text-align: center;
            cursor: pointer;
            transition: all 0.3s;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        
        .category:hover {{
            background: rgba(255,255,255,0.1);
            transform: translateY(-2px);
        }}
        
        .category-icon {{
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }}
        
        .category-name {{
            font-weight: 600;
            margin-bottom: 0.25rem;
        }}
        
        .category-count {{
            font-size: 0.875rem;
            color: #888;
        }}
        
        .section {{
            margin-bottom: 3rem;
        }}
        
        .section-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
        }}
        
        .section-title {{
            font-size: 1.5rem;
            font-weight: 600;
        }}
        
        .view-all {{
            color: #667eea;
            text-decoration: none;
            font-size: 0.875rem;
        }}
        
        .plugins-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 1.5rem;
        }}
        
        .plugin-card {{
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 1.5rem;
            border: 1px solid rgba(255,255,255,0.1);
            transition: all 0.3s;
        }}
        
        .plugin-card:hover {{
            background: rgba(255,255,255,0.08);
            transform: translateY(-2px);
        }}
        
        .plugin-header {{
            display: flex;
            align-items: flex-start;
            gap: 1rem;
            margin-bottom: 1rem;
        }}
        
        .plugin-icon {{
            width: 48px;
            height: 48px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.5rem;
        }}
        
        .plugin-info {{
            flex: 1;
        }}
        
        .plugin-name {{
            font-weight: 600;
            font-size: 1.1rem;
            margin-bottom: 0.25rem;
        }}
        
        .plugin-author {{
            font-size: 0.875rem;
            color: #888;
        }}
        
        .plugin-description {{
            color: #a0a0a0;
            font-size: 0.9rem;
            line-height: 1.5;
            margin-bottom: 1rem;
        }}
        
        .plugin-footer {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.875rem;
        }}
        
        .plugin-stats {{
            display: flex;
            gap: 1rem;
            color: #888;
        }}
        
        .plugin-rating {{
            color: #ffd700;
        }}
        
        .install-btn {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.875rem;
            transition: opacity 0.3s;
        }}
        
        .install-btn:hover {{
            opacity: 0.9;
        }}
        
        .badge {{
            display: inline-block;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: 600;
            margin-left: 0.5rem;
        }}
        
        .badge-official {{
            background: #4CAF50;
            color: #fff;
        }}
        
        .badge-verified {{
            background: #2196F3;
            color: #fff;
        }}
        
        footer {{
            text-align: center;
            padding: 3rem 0;
            color: #666;
            border-top: 1px solid rgba(255,255,255,0.1);
            margin-top: 3rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🔌 Claude Code Plugin Marketplace</h1>
            <p class="subtitle">Discover and install plugins to extend Claude Code's capabilities</p>
            
            <div class="search-box">
                <input type="text" placeholder="Search plugins..." id="searchInput">
            </div>
        </header>
        
        <div class="categories">
            <div class="category">
                <div class="category-icon">🛠️</div>
                <div class="category-name">Tools</div>
                <div class="category-count">45 plugins</div>
            </div>
            <div class="category">
                <div class="category-icon">🔌</div>
                <div class="category-name">Integrations</div>
                <div class="category-count">32 plugins</div>
            </div>
            <div class="category">
                <div class="category-icon">🎨</div>
                <div class="category-name">Themes</div>
                <div class="category-count">18 plugins</div>
            </div>
            <div class="category">
                <div class="category-icon">🌐</div>
                <div class="category-name">Languages</div>
                <div class="category-count">24 plugins</div>
            </div>
            <div class="category">
                <div class="category-icon">⚡</div>
                <div class="category-name">Productivity</div>
                <div class="category-count">28 plugins</div>
            </div>
            <div class="category">
                <div class="category-icon">💻</div>
                <div class="category-name">Development</div>
                <div class="category-count">9 plugins</div>
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">🔥 Trending</h2>
                <a href="/search?sort=downloads" class="view-all">View all →</a>
            </div>
            <div class="plugins-grid" id="trendingPlugins">
                <!-- Plugins will be loaded here -->
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">✨ New Arrivals</h2>
                <a href="/search?sort=newest" class="view-all">View all →</a>
            </div>
            <div class="plugins-grid" id="newPlugins">
                <!-- Plugins will be loaded here -->
            </div>
        </div>
        
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">⭐ Official Plugins</h2>
                <a href="/search?filter=official" class="view-all">View all →</a>
            </div>
            <div class="plugins-grid" id="officialPlugins">
                <!-- Plugins will be loaded here -->
            </div>
        </div>
        
        <footer>
            <p>Claude Code Plugin Marketplace v{}</p>
            <p style="margin-top: 0.5rem; font-size: 0.875rem;">
                <a href="/api/health" style="color: #667eea;">API Health</a> • 
                <a href="/api/docs" style="color: #667eea;">API Docs</a>
            </p>
        </footer>
    </div>
    
    <script>
        // Load featured plugins
        async function loadFeatured() {{
            try {{
                const response = await fetch('/api/featured');
                const data = await response.json();
                
                if (data.success) {{
                    renderPlugins('trendingPlugins', data.data.trending);
                    renderPlugins('newPlugins', data.data.newest);
                    renderPlugins('officialPlugins', data.data.official);
                }}
            }} catch (error) {{
                console.error('Failed to load plugins:', error);
            }}
        }}
        
        function renderPlugins(containerId, plugins) {{
            const container = document.getElementById(containerId);
            container.innerHTML = plugins.map(plugin => `
                <div class="plugin-card">
                    <div class="plugin-header">
                        <div class="plugin-icon">🔌</div>
                        <div class="plugin-info">
                            <div class="plugin-name">
                                ${{plugin.name}}
                                ${{plugin.is_official ? '<span class="badge badge-official">Official</span>' : ''}}
                                ${{plugin.is_verified ? '<span class="badge badge-verified">Verified</span>' : ''}}
                            </div>
                            <div class="plugin-author">by ${{plugin.author}}</div>
                        </div>
                    </div>
                    <div class="plugin-description">${{plugin.description}}</div>
                    <div class="plugin-footer">
                        <div class="plugin-stats">
                            <span>⬇️ ${{formatNumber(plugin.downloads)}}</span>
                            <span class="plugin-rating">⭐ ${{plugin.rating}}</span>
                        </div>
                        <button class="install-btn" onclick="installPlugin('${{plugin.id}}')">Install</button>
                    </div>
                </div>
            `).join('');
        }}
        
        function formatNumber(num) {{
            if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
            if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
            return num.toString();
        }}
        
        async function installPlugin(pluginId) {{
            try {{
                const response = await fetch(`/api/plugins/${{pluginId}}/install`, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ plugin_id: pluginId }})
                }});
                const data = await response.json();
                
                if (data.success) {{
                    alert('Plugin installed successfully!');
                }} else {{
                    alert('Failed to install plugin: ' + data.error);
                }}
            }} catch (error) {{
                alert('Failed to install plugin: ' + error.message);
            }}
        }}
        
        // Search functionality
        document.getElementById('searchInput').addEventListener('keypress', function(e) {{
            if (e.key === 'Enter') {{
                window.location.href = '/search?q=' + encodeURIComponent(this.value);
            }}
        }});
        
        // Load plugins on page load
        loadFeatured();
    </script>
</body>
</html>"#,
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Render the plugin detail page
    pub fn render_plugin_detail(&self, plugin_id: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Plugin Details - Claude Code Marketplace</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            color: #fff;
        }}
        
        .container {{
            max-width: 1000px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .back-link {{
            color: #667eea;
            text-decoration: none;
            margin-bottom: 2rem;
            display: inline-block;
        }}
        
        .plugin-header {{
            display: flex;
            gap: 2rem;
            margin-bottom: 3rem;
            padding: 2rem;
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
        }}
        
        .plugin-icon-large {{
            width: 120px;
            height: 120px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 24px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 3rem;
        }}
        
        .plugin-info {{
            flex: 1;
        }}
        
        .plugin-name {{
            font-size: 2rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }}
        
        .plugin-meta {{
            color: #888;
            margin-bottom: 1rem;
        }}
        
        .plugin-stats {{
            display: flex;
            gap: 2rem;
            margin-bottom: 1.5rem;
        }}
        
        .stat {{
            text-align: center;
        }}
        
        .stat-value {{
            font-size: 1.5rem;
            font-weight: 600;
            color: #fff;
        }}
        
        .stat-label {{
            font-size: 0.875rem;
            color: #888;
        }}
        
        .install-btn-large {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #fff;
            border: none;
            padding: 1rem 2rem;
            border-radius: 8px;
            font-size: 1.1rem;
            cursor: pointer;
            transition: opacity 0.3s;
        }}
        
        .install-btn-large:hover {{
            opacity: 0.9;
        }}
        
        .section {{
            margin-bottom: 2rem;
            padding: 2rem;
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
        }}
        
        .section-title {{
            font-size: 1.25rem;
            font-weight: 600;
            margin-bottom: 1rem;
        }}
        
        .section-content {{
            color: #a0a0a0;
            line-height: 1.8;
        }}
        
        .tags {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.5rem;
        }}
        
        .tag {{
            background: rgba(102, 126, 234, 0.2);
            color: #667eea;
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.875rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/" class="back-link">← Back to Marketplace</a>
        
        <div class="plugin-header">
            <div class="plugin-icon-large">🔌</div>
            <div class="plugin-info">
                <h1 class="plugin-name">{}</h1>
                <div class="plugin-meta">Loading plugin details...</div>
                <div class="plugin-stats">
                    <div class="stat">
                        <div class="stat-value" id="downloads">-</div>
                        <div class="stat-label">Downloads</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value" id="rating">-</div>
                        <div class="stat-label">Rating</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value" id="version">-</div>
                        <div class="stat-label">Version</div>
                    </div>
                </div>
                <button class="install-btn-large" onclick="installPlugin('{}')">Install Plugin</button>
            </div>
        </div>
        
        <div class="section">
            <h2 class="section-title">Description</h2>
            <div class="section-content" id="description">
                Loading...
            </div>
        </div>
        
        <div class="section">
            <h2 class="section-title">Tags</h2>
            <div class="tags" id="tags">
                Loading...
            </div>
        </div>
    </div>
    
    <script>
        const pluginId = '{}';
        
        async function loadPluginDetails() {{
            try {{
                const response = await fetch(`/api/plugins/${{pluginId}}`);
                const data = await response.json();
                
                if (data.success) {{
                    const plugin = data.data;
                    document.querySelector('.plugin-name').textContent = plugin.name;
                    document.querySelector('.plugin-meta').textContent = `by ${{plugin.author}} • ${{plugin.license}} license`;
                    document.getElementById('downloads').textContent = formatNumber(plugin.downloads);
                    document.getElementById('rating').textContent = '⭐ ' + plugin.rating;
                    document.getElementById('version').textContent = plugin.version;
                    document.getElementById('description').textContent = plugin.description;
                    document.getElementById('tags').innerHTML = plugin.tags.map(tag => 
                        `<span class="tag">${{tag}}</span>`
                    ).join('');
                }}
            }} catch (error) {{
                console.error('Failed to load plugin details:', error);
            }}
        }}
        
        function formatNumber(num) {{
            if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
            if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
            return num.toString();
        }}
        
        async function installPlugin(pluginId) {{
            try {{
                const response = await fetch(`/api/plugins/${{pluginId}}/install`, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ plugin_id: pluginId }})
                }});
                const data = await response.json();
                
                if (data.success) {{
                    alert('Plugin installed successfully!');
                }} else {{
                    alert('Failed to install plugin: ' + data.error);
                }}
            }} catch (error) {{
                alert('Failed to install plugin: ' + error.message);
            }}
        }}
        
        loadPluginDetails();
    </script>
</body>
</html>"#,
            plugin_id, plugin_id, plugin_id
        )
    }

    /// Render the search page
    pub fn render_search(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Search Plugins - Claude Code Marketplace</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            color: #fff;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .search-header {{
            margin-bottom: 2rem;
        }}
        
        .back-link {{
            color: #667eea;
            text-decoration: none;
            margin-bottom: 1rem;
            display: inline-block;
        }}
        
        .search-box {{
            display: flex;
            gap: 1rem;
            margin-bottom: 1.5rem;
        }}
        
        .search-box input {{
            flex: 1;
            padding: 1rem 1.5rem;
            font-size: 1.1rem;
            border: 2px solid rgba(255,255,255,0.1);
            border-radius: 8px;
            background: rgba(255,255,255,0.05);
            color: #fff;
            outline: none;
        }}
        
        .search-box input:focus {{
            border-color: #667eea;
        }}
        
        .filters {{
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            margin-bottom: 2rem;
        }}
        
        .filter select {{
            padding: 0.5rem 1rem;
            background: rgba(255,255,255,0.05);
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 6px;
            color: #fff;
            outline: none;
        }}
        
        .results-count {{
            color: #888;
            margin-bottom: 1.5rem;
        }}
        
        .plugins-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 1.5rem;
        }}
        
        .plugin-card {{
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 1.5rem;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        
        .plugin-header {{
            display: flex;
            align-items: flex-start;
            gap: 1rem;
            margin-bottom: 1rem;
        }}
        
        .plugin-icon {{
            width: 48px;
            height: 48px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.5rem;
        }}
        
        .plugin-info {{
            flex: 1;
        }}
        
        .plugin-name {{
            font-weight: 600;
            font-size: 1.1rem;
        }}
        
        .plugin-author {{
            font-size: 0.875rem;
            color: #888;
        }}
        
        .plugin-description {{
            color: #a0a0a0;
            font-size: 0.9rem;
            margin-bottom: 1rem;
        }}
        
        .plugin-footer {{
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        
        .plugin-stats {{
            display: flex;
            gap: 1rem;
            color: #888;
            font-size: 0.875rem;
        }}
        
        .pagination {{
            display: flex;
            justify-content: center;
            gap: 0.5rem;
            margin-top: 3rem;
        }}
        
        .page-btn {{
            padding: 0.5rem 1rem;
            background: rgba(255,255,255,0.05);
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 6px;
            color: #fff;
            cursor: pointer;
        }}
        
        .page-btn.active {{
            background: #667eea;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="search-header">
            <a href="/" class="back-link">← Back to Marketplace</a>
            
            <div class="search-box">
                <input type="text" id="searchInput" placeholder="Search plugins...">
            </div>
            
            <div class="filters">
                <div class="filter">
                    <select id="categoryFilter">
                        <option value="">All Categories</option>
                        <option value="tools">Tools</option>
                        <option value="integrations">Integrations</option>
                        <option value="themes">Themes</option>
                        <option value="language-support">Language Support</option>
                        <option value="productivity">Productivity</option>
                        <option value="development">Development</option>
                    </select>
                </div>
                <div class="filter">
                    <select id="sortFilter">
                        <option value="relevance">Relevance</option>
                        <option value="downloads">Most Downloads</option>
                        <option value="rating">Highest Rated</option>
                        <option value="newest">Newest</option>
                        <option value="updated">Recently Updated</option>
                    </select>
                </div>
            </div>
            
            <div class="results-count" id="resultsCount">Loading...</div>
        </div>
        
        <div class="plugins-grid" id="results">
            <!-- Results will be loaded here -->
        </div>
        
        <div class="pagination" id="pagination">
            <!-- Pagination will be loaded here -->
        </div>
    </div>
    
    <script>
        let currentPage = 1;
        let currentQuery = '';
        
        async function searchPlugins(page = 1) {{
            const query = document.getElementById('searchInput').value;
            const category = document.getElementById('categoryFilter').value;
            const sort = document.getElementById('sortFilter').value;
            
            currentQuery = query;
            currentPage = page;
            
            try {{
                let url = `/api/plugins?page=${{page}}`;
                if (query) url += `&q=${{encodeURIComponent(query)}}`;
                if (category) url += `&category=${{category}}`;
                if (sort) url += `&sort=${{sort}}`;
                
                const response = await fetch(url);
                const data = await response.json();
                
                if (data.success) {{
                    renderResults(data.data);
                }}
            }} catch (error) {{
                console.error('Search failed:', error);
            }}
        }}
        
        function renderResults(data) {{
            document.getElementById('resultsCount').textContent = 
                `${{data.total}} plugin${{data.total !== 1 ? 's' : ''}} found`;
            
            const container = document.getElementById('results');
            container.innerHTML = data.plugins.map(plugin => `
                <div class="plugin-card">
                    <div class="plugin-header">
                        <div class="plugin-icon">🔌</div>
                        <div class="plugin-info">
                            <div class="plugin-name">${{plugin.name}}</div>
                            <div class="plugin-author">by ${{plugin.author}}</div>
                        </div>
                    </div>
                    <div class="plugin-description">${{plugin.description}}</div>
                    <div class="plugin-footer">
                        <div class="plugin-stats">
                            <span>⬇️ ${{formatNumber(plugin.downloads)}}</span>
                            <span>⭐ ${{plugin.rating}}</span>
                        </div>
                        <a href="/plugin/${{plugin.id}}" style="color: #667eea; text-decoration: none;">Details →</a>
                    </div>
                </div>
            `).join('');
            
            // Render pagination
            renderPagination(data);
        }}
        
        function renderPagination(data) {{
            const container = document.getElementById('pagination');
            let html = '';
            
            for (let i = 1; i <= data.total_pages; i++) {{
                html += `<button class="page-btn ${{i === data.page ? 'active' : ''}}" onclick="searchPlugins(${{i}})">${{i}}</button>`;
            }}
            
            container.innerHTML = html;
        }}
        
        function formatNumber(num) {{
            if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
            if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
            return num.toString();
        }}
        
        // Event listeners
        document.getElementById('searchInput').addEventListener('keypress', function(e) {{
            if (e.key === 'Enter') searchPlugins(1);
        }});
        
        document.getElementById('categoryFilter').addEventListener('change', () => searchPlugins(1));
        document.getElementById('sortFilter').addEventListener('change', () => searchPlugins(1));
        
        // Initial search
        searchPlugins();
    </script>
</body>
</html>"#
        )
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
