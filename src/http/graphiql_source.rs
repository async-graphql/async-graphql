use std::{collections::HashMap, fmt};

use askama::Template;

/// Indicates whether the user agent should send or receive user credentials
/// (cookies, basic http auth, etc.) from the other domain in the case of
/// cross-origin requests.
#[derive(Debug, Default)]
pub enum Credentials {
    /// Send user credentials if the URL is on the same origin as the calling
    /// script. This is the default value.
    #[default]
    SameOrigin,
    /// Always send user credentials, even for cross-origin calls.
    Include,
    /// Never send or receive user credentials.
    Omit,
}

impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameOrigin => write!(f, "same-origin"),
            Self::Include => write!(f, "include"),
            Self::Omit => write!(f, "omit"),
        }
    }
}

struct GraphiQLVersion<'a>(&'a str);

impl Default for GraphiQLVersion<'_> {
    fn default() -> Self {
        Self("5.2.2")
    }
}

impl fmt::Display for GraphiQLVersion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A builder for constructing a GraphiQL (v2) HTML page.
///
/// # Example
///
/// ```rust
/// use async_graphql::http::*;
///
/// GraphiQLSource::build()
///     .endpoint("/")
///     .subscription_endpoint("/ws")
///     .header("Authorization", "Bearer [token]")
///     .ws_connection_param("token", "[token]")
///     .credentials(Credentials::Include)
///     .finish();
/// ```
#[derive(Default, Template)]
#[template(path = "graphiql_source.jinja")]
pub struct GraphiQLSource<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    version: GraphiQLVersion<'a>,
    headers: Option<HashMap<&'a str, &'a str>>,
    ws_connection_params: Option<HashMap<&'a str, &'a str>>,
    title: Option<&'a str>,
    credentials: Credentials,
}

impl<'a> GraphiQLSource<'a> {
    /// Creates a builder for constructing a GraphiQL (v2) HTML page.
    pub fn build() -> GraphiQLSource<'a> {
        Default::default()
    }

    /// Sets the endpoint of the server GraphiQL will connect to.
    #[must_use]
    pub fn endpoint(self, endpoint: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource { endpoint, ..self }
    }

    /// Sets the subscription endpoint of the server GraphiQL will connect to.
    pub fn subscription_endpoint(self, endpoint: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            subscription_endpoint: Some(endpoint),
            ..self
        }
    }

    /// Sets a header to be sent with requests GraphiQL will send.
    pub fn header(self, name: &'a str, value: &'a str) -> GraphiQLSource<'a> {
        let mut headers = self.headers.unwrap_or_default();
        headers.insert(name, value);
        GraphiQLSource {
            headers: Some(headers),
            ..self
        }
    }

    /// Sets the version of GraphiQL to be fetched.
    pub fn version(self, value: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            version: GraphiQLVersion(value),
            ..self
        }
    }

    /// Sets a WS connection param to be sent during GraphiQL WS connections.
    pub fn ws_connection_param(self, name: &'a str, value: &'a str) -> GraphiQLSource<'a> {
        let mut ws_connection_params = self.ws_connection_params.unwrap_or_default();
        ws_connection_params.insert(name, value);
        GraphiQLSource {
            ws_connection_params: Some(ws_connection_params),
            ..self
        }
    }

    /// Sets the html document title.
    pub fn title(self, title: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            title: Some(title),
            ..self
        }
    }

    /// Sets credentials option for the fetch requests.
    pub fn credentials(self, credentials: Credentials) -> GraphiQLSource<'a> {
        GraphiQLSource {
            credentials,
            ..self
        }
    }

    /// Returns a GraphiQL (v2) HTML page.
    pub fn finish(self) -> String {
        self.render().expect("Failed to render template")
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_with_only_url() {
        let graphiql_source = GraphiQLSource::build().endpoint("/").finish();
        let expected = expect![[r#"
            <!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="robots" content="noindex">
                <meta name="referrer" content="origin">

    
                  <title>GraphiQL</title>
    
    
                <style>
                  body {
                    margin: 0;
                  }

                  #graphiql {
                    height: 100dvh;
                  }

                  .loading {
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 4rem;
                  }
                </style>

                <link rel="stylesheet" href="https://esm.sh/graphiql/dist/style.css" />
                <link
                  rel="stylesheet"
                  href="https://esm.sh/@graphiql/plugin-explorer/dist/style.css"
                />

                <script type="importmap">
                  {
                    "imports": {
                      "react": "https://esm.sh/react@19.1.0",
                      "react/": "https://esm.sh/react@19.1.0/",

                      "react-dom": "https://esm.sh/react-dom@19.1.0",
                      "react-dom/": "https://esm.sh/react-dom@19.1.0/",

                      "graphiql": "https://esm.sh/graphiql@5.2.2?standalone&external=react,react-dom,@graphiql/react,graphql",
                      "graphiql/": "https://esm.sh/graphiql@5.2.2/",
                      "@graphiql/plugin-explorer": "https://esm.sh/@graphiql/plugin-explorer?standalone&external=react,@graphiql/react,graphql",
                      "@graphiql/react": "https://esm.sh/@graphiql/react?standalone&external=react,react-dom,graphql,@graphiql/toolkit,@emotion/is-prop-valid",

                      "@graphiql/toolkit": "https://esm.sh/@graphiql/toolkit?standalone&external=graphql",
                      "graphql": "https://esm.sh/graphql@16.11.0",
                      "@emotion/is-prop-valid": "data:text/javascript,"
                    }
                  }
                </script>

                <script type="module">
                  import React from 'react';
                  import ReactDOM from 'react-dom/client';
                  import { GraphiQL, HISTORY_PLUGIN } from 'graphiql';
                  import { createGraphiQLFetcher } from '@graphiql/toolkit';
                  import { explorerPlugin } from '@graphiql/plugin-explorer';
                  import 'graphiql/setup-workers/esm.sh';

                  const customFetch = (url, opts = {}) => {
                    return fetch(url, {...opts, credentials: 'same-origin'})
                  }

                  const createUrl = (endpoint, subscription = false) => {
                    const url = new URL(endpoint, window.location.origin);
                    if (subscription) {
                      url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
                    }
                    return url.toString();
                  }

                  const fetcher = createGraphiQLFetcher({
                    url: createUrl('/'),
                    fetch: customFetch,
        
        
        
                  });
                  const plugins = [HISTORY_PLUGIN, explorerPlugin()];

                  function App() {
                    return React.createElement(GraphiQL, {
                      fetcher,
                      plugins,
                      defaultEditorToolsVisibility: true,
                    });
                  }

                  const container = document.getElementById('graphiql');
                  const root = ReactDOM.createRoot(container);
                  root.render(React.createElement(App));
                </script>
              </head>
              <body>
                <div id="graphiql">
                  <div class="loading">Loading…</div>
                </div>
              </body>
            </html>"#]];

        expected.assert_eq(&graphiql_source);
    }

    #[test]
    fn test_with_both_urls() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish();

        let expected = expect![[r#"
            <!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="robots" content="noindex">
                <meta name="referrer" content="origin">

    
                  <title>GraphiQL</title>
    
    
                <style>
                  body {
                    margin: 0;
                  }

                  #graphiql {
                    height: 100dvh;
                  }

                  .loading {
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 4rem;
                  }
                </style>

                <link rel="stylesheet" href="https://esm.sh/graphiql/dist/style.css" />
                <link
                  rel="stylesheet"
                  href="https://esm.sh/@graphiql/plugin-explorer/dist/style.css"
                />

                <script type="importmap">
                  {
                    "imports": {
                      "react": "https://esm.sh/react@19.1.0",
                      "react/": "https://esm.sh/react@19.1.0/",

                      "react-dom": "https://esm.sh/react-dom@19.1.0",
                      "react-dom/": "https://esm.sh/react-dom@19.1.0/",

                      "graphiql": "https://esm.sh/graphiql@5.2.2?standalone&external=react,react-dom,@graphiql/react,graphql",
                      "graphiql/": "https://esm.sh/graphiql@5.2.2/",
                      "@graphiql/plugin-explorer": "https://esm.sh/@graphiql/plugin-explorer?standalone&external=react,@graphiql/react,graphql",
                      "@graphiql/react": "https://esm.sh/@graphiql/react?standalone&external=react,react-dom,graphql,@graphiql/toolkit,@emotion/is-prop-valid",

                      "@graphiql/toolkit": "https://esm.sh/@graphiql/toolkit?standalone&external=graphql",
                      "graphql": "https://esm.sh/graphql@16.11.0",
                      "@emotion/is-prop-valid": "data:text/javascript,"
                    }
                  }
                </script>

                <script type="module">
                  import React from 'react';
                  import ReactDOM from 'react-dom/client';
                  import { GraphiQL, HISTORY_PLUGIN } from 'graphiql';
                  import { createGraphiQLFetcher } from '@graphiql/toolkit';
                  import { explorerPlugin } from '@graphiql/plugin-explorer';
                  import 'graphiql/setup-workers/esm.sh';

                  const customFetch = (url, opts = {}) => {
                    return fetch(url, {...opts, credentials: 'same-origin'})
                  }

                  const createUrl = (endpoint, subscription = false) => {
                    const url = new URL(endpoint, window.location.origin);
                    if (subscription) {
                      url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
                    }
                    return url.toString();
                  }

                  const fetcher = createGraphiQLFetcher({
                    url: createUrl('/'),
                    fetch: customFetch,
        
                    subscriptionUrl: createUrl('/ws'),
        
        
        
                  });
                  const plugins = [HISTORY_PLUGIN, explorerPlugin()];

                  function App() {
                    return React.createElement(GraphiQL, {
                      fetcher,
                      plugins,
                      defaultEditorToolsVisibility: true,
                    });
                  }

                  const container = document.getElementById('graphiql');
                  const root = ReactDOM.createRoot(container);
                  root.render(React.createElement(App));
                </script>
              </head>
              <body>
                <div id="graphiql">
                  <div class="loading">Loading…</div>
                </div>
              </body>
            </html>"#]];

        expected.assert_eq(&graphiql_source);
    }

    #[test]
    fn test_with_all_options() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .header("Authorization", "Bearer [token]")
            .version("3.9.0")
            .ws_connection_param("token", "[token]")
            .title("Awesome GraphiQL IDE Test")
            .credentials(Credentials::Include)
            .finish();

        let expected = expect![[r#"
            <!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="robots" content="noindex">
                <meta name="referrer" content="origin">

    
                  <title>Awesome GraphiQL IDE Test</title>
    
    
                <style>
                  body {
                    margin: 0;
                  }

                  #graphiql {
                    height: 100dvh;
                  }

                  .loading {
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 4rem;
                  }
                </style>

                <link rel="stylesheet" href="https://esm.sh/graphiql/dist/style.css" />
                <link
                  rel="stylesheet"
                  href="https://esm.sh/@graphiql/plugin-explorer/dist/style.css"
                />

                <script type="importmap">
                  {
                    "imports": {
                      "react": "https://esm.sh/react@19.1.0",
                      "react/": "https://esm.sh/react@19.1.0/",

                      "react-dom": "https://esm.sh/react-dom@19.1.0",
                      "react-dom/": "https://esm.sh/react-dom@19.1.0/",

                      "graphiql": "https://esm.sh/graphiql@3.9.0?standalone&external=react,react-dom,@graphiql/react,graphql",
                      "graphiql/": "https://esm.sh/graphiql@3.9.0/",
                      "@graphiql/plugin-explorer": "https://esm.sh/@graphiql/plugin-explorer?standalone&external=react,@graphiql/react,graphql",
                      "@graphiql/react": "https://esm.sh/@graphiql/react?standalone&external=react,react-dom,graphql,@graphiql/toolkit,@emotion/is-prop-valid",

                      "@graphiql/toolkit": "https://esm.sh/@graphiql/toolkit?standalone&external=graphql",
                      "graphql": "https://esm.sh/graphql@16.11.0",
                      "@emotion/is-prop-valid": "data:text/javascript,"
                    }
                  }
                </script>

                <script type="module">
                  import React from 'react';
                  import ReactDOM from 'react-dom/client';
                  import { GraphiQL, HISTORY_PLUGIN } from 'graphiql';
                  import { createGraphiQLFetcher } from '@graphiql/toolkit';
                  import { explorerPlugin } from '@graphiql/plugin-explorer';
                  import 'graphiql/setup-workers/esm.sh';

                  const customFetch = (url, opts = {}) => {
                    return fetch(url, {...opts, credentials: 'include'})
                  }

                  const createUrl = (endpoint, subscription = false) => {
                    const url = new URL(endpoint, window.location.origin);
                    if (subscription) {
                      url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
                    }
                    return url.toString();
                  }

                  const fetcher = createGraphiQLFetcher({
                    url: createUrl('/'),
                    fetch: customFetch,
        
                    subscriptionUrl: createUrl('/ws'),
        
        
                    headers: {
          
                      'Authorization': 'Bearer [token]',
          
                    }
        
        
                    wsConnectionParams: {
          
                      'token': '[token]',
          
                    }
        
                  });
                  const plugins = [HISTORY_PLUGIN, explorerPlugin()];

                  function App() {
                    return React.createElement(GraphiQL, {
                      fetcher,
                      plugins,
                      defaultEditorToolsVisibility: true,
                    });
                  }

                  const container = document.getElementById('graphiql');
                  const root = ReactDOM.createRoot(container);
                  root.render(React.createElement(App));
                </script>
              </head>
              <body>
                <div id="graphiql">
                  <div class="loading">Loading…</div>
                </div>
              </body>
            </html>"#]];

        expected.assert_eq(&graphiql_source);
    }
}
