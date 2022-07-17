mod errors;
use reqwest::{
    header::{HeaderValue, CONTENT_TYPE},
    Body, ClientBuilder, Method, Request, StatusCode, Url,
};
use serde::{Deserialize, Serialize};

pub use errors::AuthenticationError;

const AUTH_URL: &str = "https://iam.cloud.ibm.com/identity/token";
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenResponse {
    #[serde(rename = "access_token")]
    access_token: String,
    #[serde(rename = "refresh_token")]
    refresh_token: String,
    #[serde(rename = "delegated_refresh_token")]
    delegated_refresh_token: Option<String>,
    #[serde(rename = "token_type")]
    token_type: String,
    #[serde(rename = "expires_in")]
    expires_in: i64,
    expiration: i64,
    scope: Option<String>,
}

#[allow(dead_code)]
impl TokenResponse {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn token_type(&self) -> &str {
        &self.token_type
    }

    pub fn expires_in(&self) -> i64 {
        self.expires_in
    }

    pub fn expiration(&self) -> i64 {
        self.expiration
    }

    pub fn scope(&self) -> Option<&String> {
        self.scope.as_ref()
    }

    pub fn delegated_refresh_token(&self) -> Option<&String> {
        self.delegated_refresh_token.as_ref()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Holds the IAM Access token generated by IBM Watson
pub struct IamAuthenticator {
    access_token: TokenResponse,
}

impl IamAuthenticator {
    /// Get an IAM Access token from an API key
    ///
    /// # Parameters
    ///
    /// * `api_key` - The API key for your Watson service
    ///
    /// # Example
    /// ``` no_run
    /// # use ibm_watson::auth::IamAuthenticator;
    /// # async fn foo()-> Result<(), Box<dyn std::error::Error>> {
    /// let auth = IamAuthenticator::new("api_key").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(api_key: impl AsRef<str>) -> Result<Self, AuthenticationError> {
        let url = Url::parse(AUTH_URL).unwrap();
        let mut req = Request::new(Method::POST, url);
        let headers = req.headers_mut();
        let _ = headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
        );
        let body = req.body_mut();
        *body = Some(Body::from(format!(
            "grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={}",
            api_key.as_ref()
        )));
        let client = ClientBuilder::new();
        #[cfg(feature = "http2")]
        let client = client.http2_prior_knowledge();

        let client = client.build().unwrap();
        let resp = client
            .execute(req)
            .await
            .map_err(|e| AuthenticationError::ConnectionError(e.to_string()))?;
        match resp.status() {
            StatusCode::OK => {
                // asynchronously aggregate the chunks of the body
                let access_token: TokenResponse = resp.json().await.unwrap();
                Ok(Self { access_token })
            }
            StatusCode::BAD_REQUEST => Err(AuthenticationError::ParameterValidationFailed),
            _ => unreachable!(),
        }
    }

    pub(crate) fn token_response(&self) -> &TokenResponse {
        &self.access_token
    }
}
