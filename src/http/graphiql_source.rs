/// Generate the page for GraphIQL
pub fn graphiql_source(graphql_endpoint_url: &str, subscription_endpoint: Option<&str>) -> String {
    r#"
    <html>
  <head>
    <title>Simple GraphiQL Example</title>
    <link href="https://unpkg.com/graphiql@1/graphiql.min.css" rel="stylesheet" />
  </head>
  <body style="margin: 0;">
    <div id="graphiql" style="height: 100vh;"></div>

    <script src="//unpkg.com/subscriptions-transport-ws@0.8.3/browser/client.js"></script>
    <script src="//unpkg.com/graphiql-subscriptions-fetcher@0.0.2/browser/client.js"></script>
    <script
      crossorigin
      src="https://unpkg.com/react@17/umd/react.production.min.js"
    ></script>
    <script
      crossorigin
      src="https://unpkg.com/react-dom@17/umd/react-dom.production.min.js"
    ></script>
    <script
      crossorigin
      src="https://unpkg.com/graphiql@1/graphiql.min.js"
    ></script>

    <script>
      var fetcher = graphQLParams =>
        fetch('GRAPHQL_URL', {
          method: 'post',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(graphQLParams),
        })
          .then(response => response.json())
          .catch(() => response.text());

      var subscription_url = GRAPHQL_SUBSCRIPTION_URL;

      if (subscription_url) {
        var subscriptionClient = new window.SubscriptionsTransportWs.SubscriptionClient(GRAPHQL_SUBSCRIPTION_URL, { reconnect: true });
        fetcher = window.GraphiQLSubscriptionsFetcher.graphQLFetcher(subscriptionClient, fetcher);
      }

      ReactDOM.render(
        React.createElement(GraphiQL, { fetcher }),
        document.getElementById('graphiql'),
      );
    </script>
  </body>
</html>
    "#
    .replace("GRAPHQL_URL", graphql_endpoint_url)
    .replace(
        "GRAPHQL_SUBSCRIPTION_URL",
        &match subscription_endpoint {
            Some(url) => format!("'{}'", url),
            None => "null".to_string(),
        },
    )
}
