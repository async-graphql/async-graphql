use std::collections::HashMap;

use handlebars::Handlebars;
use serde::Serialize;

use crate::http::graphiql_plugin::GraphiQLPlugin;

/// Indicates whether the user agent should send or receive user credentials
/// (cookies, basic http auth, etc.) from the other domain in the case of
/// cross-origin requests.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
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
///     .credentials(Credentials::Include)
///     .finish();
/// ```
#[derive(Default, Serialize)]
pub struct GraphiQLSource<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    headers: Option<HashMap<&'a str, &'a str>>,
    title: Option<&'a str>,
    credentials: Credentials,
    plugins: &'a [GraphiQLPlugin<'a>],
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
        let mut headers = match self.headers {
            Some(headers) => headers,
            None => HashMap::new(),
        };
        headers.insert(name, value);
        GraphiQLSource {
            headers: Some(headers),
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

    /// Sets plugins
    pub fn plugins(self, plugins: &'a [GraphiQLPlugin]) -> GraphiQLSource<'a> {
        GraphiQLSource { plugins, ..self }
    }

    /// Returns a GraphiQL (v2) HTML page.
    pub fn finish(self) -> String {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string(
                "graphiql_v2_source",
                include_str!("./graphiql_v2_source.hbs"),
            )
            .expect("Failed to register template");

        handlebars
            .render("graphiql_v2_source", &self)
            .expect("Failed to render template")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_only_url() {
        let graphiql_source = GraphiQLSource::build().endpoint("/").finish();

        assert_eq!(
            graphiql_source,
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="referrer" content="origin">

    <title>GraphiQL IDE</title>

    <style>
      body {
        height: 100%;
        margin: 0;
        width: 100%;
        overflow: hidden;
      }

      #graphiql {
        height: 100vh;
      }
    </style>
    <script
      crossorigin
      src="https://unpkg.com/react@17/umd/react.development.js"
    ></script>
    <script
      crossorigin
      src="https://unpkg.com/react-dom@17/umd/react-dom.development.js"
    ></script>
    <link rel="icon" href="https://graphql.org/favicon.ico">
    <link rel="stylesheet" href="https://unpkg.com/graphiql/graphiql.min.css" />
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    <script
      src="https://unpkg.com/graphiql/graphiql.min.js"
      type="application/javascript"
    ></script>
    <script>
      customFetch = (url, opts = {}) => {
        return fetch(url, {...opts, credentials: 'same-origin'})
      }

      createUrl = (endpoint, subscription = false) => {
        const url = new URL(endpoint, window.location.origin);
        if (subscription) {
          url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
        }
        return url.toString();
      }

      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: createUrl('/'),
            fetch: customFetch,
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>"#
        )
    }

    #[test]
    fn test_with_both_urls() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish();

        assert_eq!(
            graphiql_source,
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="referrer" content="origin">

    <title>GraphiQL IDE</title>

    <style>
      body {
        height: 100%;
        margin: 0;
        width: 100%;
        overflow: hidden;
      }

      #graphiql {
        height: 100vh;
      }
    </style>
    <script
      crossorigin
      src="https://unpkg.com/react@17/umd/react.development.js"
    ></script>
    <script
      crossorigin
      src="https://unpkg.com/react-dom@17/umd/react-dom.development.js"
    ></script>
    <link rel="icon" href="https://graphql.org/favicon.ico">
    <link rel="stylesheet" href="https://unpkg.com/graphiql/graphiql.min.css" />
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    <script
      src="https://unpkg.com/graphiql/graphiql.min.js"
      type="application/javascript"
    ></script>
    <script>
      customFetch = (url, opts = {}) => {
        return fetch(url, {...opts, credentials: 'same-origin'})
      }

      createUrl = (endpoint, subscription = false) => {
        const url = new URL(endpoint, window.location.origin);
        if (subscription) {
          url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
        }
        return url.toString();
      }

      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: createUrl('/'),
            fetch: customFetch,
            subscriptionUrl: createUrl('/ws', true),
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>"#
        )
    }

    #[test]
    fn test_with_all_options() {
        use crate::http::graphiql_plugin_explorer;
        let graphiql_source = GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .header("Authorization", "Bearer [token]")
            .title("Awesome GraphiQL IDE Test")
            .credentials(Credentials::Include)
            .plugins(&[graphiql_plugin_explorer()])
            .finish();

        assert_eq!(
            graphiql_source,
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="referrer" content="origin">

    <title>Awesome GraphiQL IDE Test</title>

    <style>
      body {
        height: 100%;
        margin: 0;
        width: 100%;
        overflow: hidden;
      }

      #graphiql {
        height: 100vh;
      }
    </style>
    <script
      crossorigin
      src="https://unpkg.com/react@17/umd/react.development.js"
    ></script>
    <script
      crossorigin
      src="https://unpkg.com/react-dom@17/umd/react-dom.development.js"
    ></script>
    <link rel="icon" href="https://graphql.org/favicon.ico">
    <link rel="stylesheet" href="https://unpkg.com/graphiql/graphiql.min.css" />
    <link rel="stylesheet" href="https://unpkg.com/@graphiql/plugin-explorer/dist/style.css" />
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    <script
      src="https://unpkg.com/graphiql/graphiql.min.js"
      type="application/javascript"
    ></script>
    <script
      src="https://unpkg.com/@graphiql/plugin-explorer/dist/index.umd.js"
      crossorigin
    ></script>
    <script>
      customFetch = (url, opts = {}) => {
        return fetch(url, {...opts, credentials: 'include'})
      }

      createUrl = (endpoint, subscription = false) => {
        const url = new URL(endpoint, window.location.origin);
        if (subscription) {
          url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
        }
        return url.toString();
      }

      const plugins = [];
      plugins.push(GraphiQLPluginExplorer.explorerPlugin());

      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: createUrl('/'),
            fetch: customFetch,
            subscriptionUrl: createUrl('/ws', true),
            headers: {
              'Authorization': 'Bearer [token]',
            },
          }),
          defaultEditorToolsVisibility: true,
          plugins,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>"#
        )
    }
}
