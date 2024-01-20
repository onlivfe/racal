//! An optional API client feature using `reqwest`
//!
//! Besides using this, you could instead easily implement your own client using
//! a different HTTP library with the [`racal::Queryable`](crate::Queryable)
//! trait.

use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{FromApiState, Queryable, RequestMethod};

/// An error that may happen with an API query
#[derive(Debug, Error)]
pub enum ApiError {
	/// An error happened with serialization
	#[error("An error happened with serialization: {0}")]
	Serde(serde_json::Error),

	/// An error happened with the request itself
	#[error("An error happened with the request itself: {0}")]
	Reqwest(reqwest::Error),
}

impl From<serde_json::Error> for ApiError {
	fn from(err: serde_json::Error) -> Self { Self::Serde(err) }
}

impl From<reqwest::Error> for ApiError {
	fn from(err: reqwest::Error) -> Self { Self::Reqwest(err) }
}

/// An API client that can be used to create queries
#[async_trait::async_trait]
pub trait ApiClient<State> {
	/// Gets the API state
	fn state(&self) -> &State;

	/// Gets the actual reqwest client
	fn client(&self) -> &Client;

	/// A way to modify the request right before sending it
	///
	/// Can also for example be used to implement rate limits thanks to the async
	/// nature
	async fn before_request(
		&self, req: RequestBuilder,
	) -> Result<RequestBuilder, ApiError> {
		Ok(req)
	}

	/// A way to modify the request after it's been received
	///
	/// By default errors on bad status messages and just deserializes the value,
	/// using the queryable.
	///
	/// Can also for example be used to implement rate limits thanks to the async
	/// nature.
	async fn handle_response<ReturnType, FromState, QueryableType>(
		&self, queryable: QueryableType, response: Response,
	) -> Result<ReturnType, ApiError>
	where
		ReturnType: DeserializeOwned,
		FromState: FromApiState<State>,
		QueryableType: Queryable<FromState, ReturnType> + Send + Sync,
	{
		let response = response.error_for_status()?;
		let val = response.bytes().await?;
		Ok(queryable.deserialize(&val)?)
	}

	/// Creates a query
	async fn query<ReturnType, FromState, QueryableType>(
		&self, queryable: QueryableType,
	) -> Result<ReturnType, ApiError>
	where
		ReturnType: DeserializeOwned,
		FromState: FromApiState<State>,
		QueryableType: Queryable<FromState, ReturnType> + Send + Sync,
	{
		let request = Self::build_request(
			self.client(),
			FromState::from_state(self.state()),
			&queryable,
		)?;
		let request = self.before_request(request).await?;
		let response = request.send().await?;

		self.handle_response(queryable, response).await
	}

	/// Builds the base request
	fn build_request<ReturnType, FromState, QueryableType>(
		http: &Client, api_state: &FromState, queryable: &QueryableType,
	) -> Result<RequestBuilder, ApiError>
	where
		ReturnType: DeserializeOwned,
		FromState: FromApiState<State>,
		QueryableType: Queryable<FromState, ReturnType> + Send + Sync,
	{
		let mut request = http.request(
			match queryable.method(api_state) {
				RequestMethod::Get => reqwest::Method::GET,
				RequestMethod::Head => reqwest::Method::HEAD,
				RequestMethod::Patch => reqwest::Method::PATCH,
				RequestMethod::Post => reqwest::Method::POST,
				RequestMethod::Put => reqwest::Method::PUT,
				RequestMethod::Delete => reqwest::Method::DELETE,
			},
			queryable.url(api_state),
		);
		if let Some(body) = queryable.body(api_state) {
			request = request.body(body?).header("Content-Type", "application/json");
		}

		Ok(request)
	}
}
