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
///     .finish();
/// ```
#[derive(Default)]
pub struct GraphiQLSource<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    headers: Option<HashMap<&'a str, &'a str>>,
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
    <link rel="icon" href="https://graphql.org/favicon.ico">
    <link rel="stylesheet" href="https://unpkg.com/graphiql@2.0.3/graphiql.min.css" integrity="sha512-GePWbFGQjhytVrlixKmga7iqaUN6EZWyWhYRMUWaug+1a4SNMU9Zl5OUC6r1jHxXv/M3llhlw7Ilm7gsfyx+nw==" crossorigin="anonymous" referrerpolicy="no-referrer"/>
  </head>

  <body>
    <div id="graphiql">Loading...</div>
    
    <script src="https://unpkg.com/react@18.2.0/umd/react.production.min.js" integrity="sha512-8Q6Y9XnTbOE+JNvjBQwJ2H8S+UV4uA6hiRykhdtIyDYZ2TprdNmWOUaKdGzOhyr4dCyk287OejbPvwl7lrfqrQ==" crossorigin="anonymous"  referrerpolicy="no-referrer"></script>
    <script src="https://unpkg.com/react-dom@18.2.0/umd/react-dom.production.min.js" integrity="sha512-MOCpqoRoisCTwJ8vQQiciZv0qcpROCidek3GTFS6KTk2+y7munJIlKCVkFCYY+p3ErYFXCjmFjnfTTRSC1OHWQ==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
    <script src="https://unpkg.com/graphiql@2.0.3/graphiql.min.js" integrity="sha512-2u/R6n1Z1Y2sWQ6IYlWqnzI6uz7xZTaI+RQCftaAhPTSkc01er59mQLMdYb31xePGWulyPebn8pfetF49kBU6Q==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
    
    <script>
      ReactDOM.render(
        React.createElement(GraphiQL, {
          fetcher: GraphiQL.createFetcher({
            url: %GRAPHIQL_URL%,
            subscriptionUrl: %GRAPHIQL_SUBSCRIPTION_URL%,
            headers: %GRAPHIQL_HEADERS%,
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
