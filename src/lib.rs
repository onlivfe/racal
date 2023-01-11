//! REST API client abstraction library.
//!
//! This is a very minimal crate

#![cfg_attr(nightly, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(unsafe_code)]
#![deny(clippy::cargo)]
#![warn(missing_docs)]
#![deny(rustdoc::invalid_html_tags)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// My project my choice, tabs are literally made for indentation, spaces not.
#![allow(clippy::tabs_in_doc_comments)]

use serde::de::DeserializeOwned;

/// Possible request methods
#[derive(Debug, Clone)]
pub enum RequestMethod {
	/// The request should use the `HEAD` method
	Head,
	/// The request should use the `GET` method
	Get,
	/// The request should use the `POST` method
	Post,
	/// The request should use the `PUT` method
	Put,
	/// The request should use the `PATCH` method
	Patch,
	/// The request should use the `DELETE` method
	Delete,
}

/// Data for a HTTP request & response
pub trait Queryable<RequiredApiState, ResponseType: DeserializeOwned> {
	/// The URL of the request
	fn url(&self) -> String;

	/// The method to use for the request
	///
	/// Defaults to `GET`.
	fn method(&self) -> RequestMethod {
		RequestMethod::Get
	}

	/// Creates a JSON body for the request
	///
	/// Defaults to no body.
	fn body(&self) -> Option<serde_json::Result<Vec<u8>>> {
		None
	}
}
