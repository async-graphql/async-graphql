use std::collections::HashMap;

use handlebars::Handlebars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A builder for constructing an Altair HTML page.
#[derive(Default, Serialize)]
pub struct AltairSource<'a> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}
impl<'a> AltairSource<'a> {
    /// Creates a builder for constructing an Altair HTML page.
    pub fn build() -> AltairSource<'a> {
        Default::default()
    }

    /// Sets the html document title.
    pub fn title(self, title: &'a str) -> AltairSource<'a> {
        AltairSource {
            title: Some(title),
            ..self
        }
    }

    /// Sets the [Altair options](https://github.com/altair-graphql/altair?tab=readme-ov-file#configuration-options).
    ///
    /// # Examples
    ///
    /// With on-the-fly options:
    /// ```rust
    /// use async_graphql::http::*;
    /// use serde_json::json;
    ///
    /// AltairSource::build()
    ///     .options(json!({
    ///         "endpointURL": "/",
    ///         "subscriptionsEndpoint": "/ws",
    ///         "subscriptionsProtocol": "wss",
    ///     }))
    ///     .finish();
    /// ```
    ///
    /// With strongly-typed [AltairConfigOptions], useful when reading options
    /// from config files: ```rust
    /// use async_graphql::http::*;
    ///
    /// AltairSource::build()
    ///     .options(AltairConfigOptions {
    ///         window_options: Some(AltairWindowOptions {
    ///             endpoint_url: Some("/".to_owned()),
    ///             subscriptions_endpoint: Some("/ws".to_owned()),
    ///             subscriptions_protocol: Some("wss".to_owned()),
    ///             ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     })
    ///     .finish();
    /// ```
    pub fn options<T: Serialize>(self, options: T) -> AltairSource<'a> {
        AltairSource {
            options: Some(serde_json::to_value(options).expect("Failed to serialize options")),
            ..self
        }
    }

    /// Returns an Altair HTML page.
    pub fn finish(self) -> String {
        let mut handlebars = Handlebars::new();

        handlebars.register_helper("toJson", Box::new(ToJsonHelper));

        handlebars
            .register_template_string("altair_source", include_str!("./altair_source.hbs"))
            .expect("Failed to register template");

        handlebars
            .render("altair_source", &self)
            .expect("Failed to render template")
    }
}

/// Altair window [options](https://github.com/altair-graphql/altair/blob/master/packages/altair-core/src/config.ts#L10)
#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AltairWindowOptions {
    /// Initial name of the window
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_name: Option<String>,
    /// URL to set as the server endpoint
    #[serde(
        rename = "endpointURL",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint_url: Option<String>,
    /// URL to set as the subscription endpoint. This can be relative or
    /// absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscriptions_endpoint: Option<String>,
    /// URL protocol for the subscription endpoint. This is used if the
    /// specified subscriptions endpoint is relative.
    ///
    /// e.g. wss
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscriptions_protocol: Option<String>,
    /// Initial query to be added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_query: Option<String>,
    /// Initial variables to be added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_variables: Option<String>,
    /// Initial pre-request script to be added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_pre_request_script: Option<String>,
    /// Initial post-request script to be added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_post_request_script: Option<String>,
    /// Initial authorization type and data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_authorization: Option<AltairAuthorizationProviderInput>,
    /// Initial headers object to be added
    /// ```js
    /// {
    ///  'X-GraphQL-Token': 'asd7-237s-2bdk-nsdk4'
    /// }
    /// ```
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub initial_headers: HashMap<String, String>,
    /// Initial subscriptions connection params
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub initial_subscriptions_payload: HashMap<String, String>,
    /// HTTP method to use for making requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_http_method: Option<AltairHttpVerb>,
}

/// Altair config [options](https://github.com/altair-graphql/altair/blob/master/packages/altair-core/src/config.ts#L79)
#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AltairConfigOptions {
    /// Options to be applied on every new window (including the initial)
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub window_options: Option<AltairWindowOptions>,
    /// Initial Environments to be added
    /// ```js
    ///  {
    ///    base: {
    ///     title: 'Environment',
    ///     variables: {}
    ///   },
    ///   subEnvironments: [
    ///     {
    ///       title: 'sub-1',
    ///       variables: {}
    ///     }
    ///   ]
    /// }
    /// ```
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_environments: Option<AltairInitialEnvironments>,
    /// Namespace for storing the data for the altair instance.
    ///
    /// Use this when you have multiple altair instances running on the same
    /// domain.
    ///
    /// e.g. altair_dev_
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_storage_namespace: Option<String>,
    /// Initial app settings to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_settings: Option<AltairSettingsState>,
    /// Indicates if the state should be preserved for subsequent app loads
    /// (default true)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preserve_state: Option<bool>,
    /// List of options for windows to be loaded
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_windows: Vec<AltairWindowOptions>,
    /// Persisted settings for the app. The settings will be merged with the app
    /// settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persisted_settings: Option<AltairSettingsState>,
    /// Disable the account and remote syncing functionality
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_account: Option<bool>,
}

/// Altair supported HTTP verbs
#[derive(Serialize, Deserialize, JsonSchema)]
#[allow(missing_docs)]
pub enum AltairHttpVerb {
    POST,
    GET,
    PUT,
    DELETE,
}

/// Altair initial environments setup
#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AltairInitialEnvironments {
    /// Base environment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<AltairInitialEnvironmentState>,
    /// Other sub environments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_environments: Vec<AltairInitialEnvironmentState>,
}

/// Altair initial environment state
#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AltairInitialEnvironmentState {
    /// Environment identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Environment title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, String>,
}

/// Altair authorization provider input
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum AltairAuthorizationProviderInput {
    /// Api key authorization
    #[serde(rename = "api-key")]
    ApiKey {
        /// Header name
        header_name: String,
        /// Header value
        header_value: String,
    },
    /// Basic authorization
    #[serde(rename = "basic")]
    Basic {
        /// Password
        password: String,
        /// Username
        username: String,
    },
    /// Bearer token authorization
    #[serde(rename = "bearer")]
    Bearer {
        /// Token
        token: String,
    },
    /// OAuth2 access token authorization
    #[serde(rename = "oauth2")]
    OAuth2 {
        /// Access token response
        access_token_response: String,
    },
}

/// Altair application settings state
#[derive(Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AltairSettingsState {
    /// Theme
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// Theme for dark mode
    #[serde(
        rename = "theme.dark",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub theme_dark: Option<String>,
    /// Language
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<AltairSettingsLanguage>,
    /// 'Add query' functionality depth
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_query_depth_limit: Option<usize>,
    /// Editor tab size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_size: Option<usize>,
    /// Enable experimental features.
    ///
    /// Note: Might be unstable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_experimental: Option<bool>,
    /// Base Font Size
    ///
    /// default: 24
    #[serde(
        rename = "theme.fontsize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub theme_font_size: Option<usize>,
    /// Editor Font Family
    #[serde(
        rename = "theme.editorFontFamily",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub theme_editor_font_family: Option<String>,
    /// Editor Font Size
    #[serde(
        rename = "theme.editorFontSize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub theme_editor_font_size: Option<usize>,
    /// Disable push notifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_push_notification: Option<bool>,
    /// Enabled plugins
    #[serde(rename = "plugin.list", default, skip_serializing_if = "Vec::is_empty")]
    pub plugin_list: Vec<String>,
    /// Send requests with credentials (cookies)
    #[serde(
        rename = "request.withCredentials",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub request_with_credentials: Option<bool>,
    /// Reload schema on app start
    #[serde(
        rename = "schema.reloadOnStart",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub schema_reload_on_start: Option<bool>,
    /// Disable update notification
    #[serde(
        rename = "alert.disableUpdateNotification",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub alert_disable_update_notification: Option<bool>,
    /// Disable warning alerts
    #[serde(
        rename = "alert.disableWarnings",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub alert_disable_warnings: Option<bool>,
    /// Number of items allowed in history pane
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_depth: Option<usize>,
    /// Disable line numbers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_line_numbers: Option<bool>,
    /// Theme config object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme_config: Option<serde_json::Value>,
    /// Theme config object for dark mode
    #[serde(
        rename = "themeConfig.dark",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub theme_config_dark: Option<serde_json::Value>,
    /// Hides extensions object
    #[serde(
        rename = "response.hideExtensions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub response_hide_extensions: Option<bool>,
    /// Contains shortcut to action mapping
    #[serde(
        rename = "editor.shortcuts",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub editor_shortcuts: HashMap<String, String>,
    /// Disable new editor beta
    #[serde(
        rename = "beta.disable.newEditor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub beta_disable_new_editor: Option<bool>,
    /// Disable new script beta
    #[serde(
        rename = "beta.disable.newScript",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub beta_disable_new_script: Option<bool>,
    /// List of cookies to be accessible in the pre-request script
    ///
    /// e.g. ['cookie1', 'cookie2']
    #[serde(
        rename = "script.allowedCookies",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub script_allowed_cookies: Vec<String>,
    /// Enable the scrollbar in the tab list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_tablist_scrollbar: Option<bool>,
}

/// Altair supported languages
#[derive(Serialize, Deserialize, JsonSchema)]
#[allow(missing_docs)]
pub enum AltairSettingsLanguage {
    #[serde(rename = "en-US")]
    English,
    #[serde(rename = "fr-FR")]
    French,
    #[serde(rename = "es-ES")]
    Español,
    #[serde(rename = "cs-CZ")]
    Czech,
    #[serde(rename = "de-DE")]
    German,
    #[serde(rename = "pt-BR")]
    Brazilian,
    #[serde(rename = "ru-RU")]
    Russian,
    #[serde(rename = "uk-UA")]
    Ukrainian,
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
    #[serde(rename = "ja-JP")]
    Japanese,
    #[serde(rename = "sr-SP")]
    Serbian,
    #[serde(rename = "it-IT")]
    Italian,
    #[serde(rename = "pl-PL")]
    Polish,
    #[serde(rename = "ko-KR")]
    Korean,
    #[serde(rename = "ro-RO")]
    Romanian,
    #[serde(rename = "vi-VN")]
    Vietnamese,
}

struct ToJsonHelper;
impl handlebars::HelperDef for ToJsonHelper {
    #[allow(unused_assignments)]
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'rc>,
        r: &'reg handlebars::Handlebars<'reg>,
        _: &'rc handlebars::Context,
        _: &mut handlebars::RenderContext<'reg, 'rc>,
    ) -> std::result::Result<handlebars::ScopedJson<'rc>, handlebars::RenderError> {
        let mut param_idx = 0;
        let obj = h
            .param(param_idx)
            .and_then(|x| {
                if r.strict_mode() && x.is_value_missing() {
                    None
                } else {
                    Some(x.value())
                }
            })
            .ok_or_else(|| {
                handlebars::RenderErrorReason::ParamNotFoundForName("toJson", "obj".to_string())
            })
            .and_then(|x| {
                x.as_object().ok_or_else(|| {
                    handlebars::RenderErrorReason::ParamTypeMismatchForName(
                        "toJson",
                        "obj".to_string(),
                        "object".to_string(),
                    )
                })
            })?;
        param_idx += 1;
        let result = if obj.is_empty() {
            "{}".to_owned()
        } else {
            serde_json::to_string(&obj).expect("Failed to serialize json")
        };
        Ok(handlebars::ScopedJson::Derived(
            handlebars::JsonValue::from(result),
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_without_options() {
        let altair_source = AltairSource::build().title("Custom Title").finish();

        assert_eq!(
            altair_source,
            r#"<!DOCTYPE html>
<html>

  <head>
    <meta charset="utf-8">

    <title>Custom Title</title>

    <base href="https://unpkg.com/altair-static@latest/build/dist/">

    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" type="image/x-icon" href="favicon.ico">
    <link rel="stylesheet" href="styles.css">
  </head>

  <body>
    <script>
      document.addEventListener('DOMContentLoaded', () => {
        AltairGraphQL.init();
      });
    </script>
    <app-root>
      <style>
        .loading-screen {
          /*Prevents the loading screen from showing until CSS is downloaded*/
          display: none;
        }
      </style>
      <div class="loading-screen styled">
        <div class="loading-screen-inner">
          <div class="loading-screen-logo-container">
            <img src="assets/img/logo_350.svg" alt="Altair">
          </div>
          <div class="loading-screen-loading-indicator">
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
          </div>
        </div>
      </div>
    </app-root>
    <script type="text/javascript" src="runtime.js"></script>
    <script type="text/javascript" src="polyfills.js"></script>
    <script type="text/javascript" src="main.js"></script>
  </body>

</html>"#
        )
    }

    #[test]
    fn test_with_dynamic() {
        let altair_source = AltairSource::build()
            .options(json!({
                "endpointURL": "/",
                "subscriptionsEndpoint": "/ws",
            }))
            .finish();

        assert_eq!(
            altair_source,
            r#"<!DOCTYPE html>
<html>

  <head>
    <meta charset="utf-8">

    <title>Altair</title>

    <base href="https://unpkg.com/altair-static@latest/build/dist/">

    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" type="image/x-icon" href="favicon.ico">
    <link rel="stylesheet" href="styles.css">
  </head>

  <body>
    <script>
      document.addEventListener('DOMContentLoaded', () => {
        AltairGraphQL.init({"endpointURL":"/","subscriptionsEndpoint":"/ws"});
      });
    </script>
    <app-root>
      <style>
        .loading-screen {
          /*Prevents the loading screen from showing until CSS is downloaded*/
          display: none;
        }
      </style>
      <div class="loading-screen styled">
        <div class="loading-screen-inner">
          <div class="loading-screen-logo-container">
            <img src="assets/img/logo_350.svg" alt="Altair">
          </div>
          <div class="loading-screen-loading-indicator">
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
          </div>
        </div>
      </div>
    </app-root>
    <script type="text/javascript" src="runtime.js"></script>
    <script type="text/javascript" src="polyfills.js"></script>
    <script type="text/javascript" src="main.js"></script>
  </body>

</html>"#
        )
    }

    #[test]
    fn test_with_static() {
        let altair_source = AltairSource::build()
            .options(AltairConfigOptions {
                window_options: Some(AltairWindowOptions {
                    endpoint_url: Some("/".to_owned()),
                    subscriptions_endpoint: Some("/ws".to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .finish();

        assert_eq!(
            altair_source,
            r#"<!DOCTYPE html>
<html>

  <head>
    <meta charset="utf-8">

    <title>Altair</title>

    <base href="https://unpkg.com/altair-static@latest/build/dist/">

    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" type="image/x-icon" href="favicon.ico">
    <link rel="stylesheet" href="styles.css">
  </head>

  <body>
    <script>
      document.addEventListener('DOMContentLoaded', () => {
        AltairGraphQL.init({"endpointURL":"/","subscriptionsEndpoint":"/ws"});
      });
    </script>
    <app-root>
      <style>
        .loading-screen {
          /*Prevents the loading screen from showing until CSS is downloaded*/
          display: none;
        }
      </style>
      <div class="loading-screen styled">
        <div class="loading-screen-inner">
          <div class="loading-screen-logo-container">
            <img src="assets/img/logo_350.svg" alt="Altair">
          </div>
          <div class="loading-screen-loading-indicator">
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
          </div>
        </div>
      </div>
    </app-root>
    <script type="text/javascript" src="runtime.js"></script>
    <script type="text/javascript" src="polyfills.js"></script>
    <script type="text/javascript" src="main.js"></script>
  </body>

</html>"#
        )
    }
}
