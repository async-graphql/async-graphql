use std::collections::HashMap;

/// A builder for constructing a GraphiQL (v2) HTML page.
///
/// # Example
///
/// ```rust
/// use async_graphql::http::*;
///
/// GraphiQLSource::build()
///     .endpoint("http://localhost:8000")
///     .subscription_endpoint("ws://localhost:8000/ws")
///     .header("Authorization", "Bearer <token>")
///     .credentials("include")
///     .finish();
/// ```
#[derive(Default)]
pub struct GraphiQLSource<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    headers: Option<HashMap<&'a str, &'a str>>,
    title: Option<&'a str>,
    credentials: Option<&'a str>,
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
    pub fn credentials(self, credentials: &'a str) -> GraphiQLSource<'a> {
        GraphiQLSource {
            credentials: Some(credentials),
            ..self
        }
    }

    /// Returns a GraphiQL (v2) HTML page.
    pub fn finish(self) -> String {
        let graphiql_url = format!("'{}'", self.endpoint);
        let graphiql_subscription_url = self
            .subscription_endpoint
            .map(|endpoint| format!("'{}'", endpoint))
            .unwrap_or_else(|| "undefined".into());
        let graphiql_headers = match self.headers {
            Some(headers) => serde_json::to_string(&headers).unwrap(),
            None => "undefined".into(),
        };
        let graphiql_title = self.title.unwrap_or("GraphiQL IDE");
        let graphiql_credentials = self.credentials.unwrap_or("same-origin");

        r#"
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="referrer" content="origin">

    <title>%GRAPHIQL_TITLE%</title>

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
    customFetch = (...args) => {
      console.log(args);
      fetch({ ...args, credentials: "%CREDENTIALS%" })
    }

      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: %GRAPHIQL_URL%,
            subscriptionUrl: %GRAPHIQL_SUBSCRIPTION_URL%,
            headers: %GRAPHIQL_HEADERS%,
            fetch: customFetch,
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>
"#
        .replace("%GRAPHIQL_URL%", &graphiql_url)
        .replace("%GRAPHIQL_SUBSCRIPTION_URL%", &graphiql_subscription_url)
        .replace("%GRAPHIQL_HEADERS%", &graphiql_headers)
        .replace("%GRAPHIQL_TITLE%", &graphiql_title)
        .replace("%CREDENTIALS%", &graphiql_credentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_only_url() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("http://localhost:8000")
            .finish();

        assert_eq!(
            graphiql_source,
            r#"
<!DOCTYPE html>
<html>
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
      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: 'http://localhost:8000',
            subscriptionUrl: undefined,
            headers: undefined,
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>
"#
        )
    }

    #[test]
    fn test_with_both_urls() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("http://localhost:8000")
            .subscription_endpoint("ws://localhost:8000/ws")
            .finish();

        assert_eq!(
            graphiql_source,
            r#"
<!DOCTYPE html>
<html>
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
      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: 'http://localhost:8000',
            subscriptionUrl: 'ws://localhost:8000/ws',
            headers: undefined,
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>
"#
        )
    }

    #[test]
    fn test_with_all_options() {
        let graphiql_source = GraphiQLSource::build()
            .endpoint("http://localhost:8000")
            .subscription_endpoint("ws://localhost:8000/ws")
            .header("Authorization", "Bearer <token>")
            .title("Awesome GraphiQL IDE Test")
            .finish();

        assert_eq!(
            graphiql_source,
            r#"
<!DOCTYPE html>
<html>
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
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    <script
      src="https://unpkg.com/graphiql/graphiql.min.js"
      type="application/javascript"
    ></script>
    <script>
      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: 'http://localhost:8000',
            subscriptionUrl: 'ws://localhost:8000/ws',
            headers: {"Authorization":"Bearer <token>"},
          }),
          defaultEditorToolsVisibility: true,
        }),
        document.getElementById("graphiql")
      );
    </script>
  </body>
</html>
"#
        )
    }
}
