//! # Monitor core API
//!
//! Monitor core exposes an HTTP api using standard JSON serialization.
//!
//! All calls share some common HTTP params:
//! - Method: `POST`
//! - Path: `/auth`, `/read`, `/write`, `/execute`
//! - Headers:
//!   - Content-Type: `application/json`
//!   - Authorication: `your_jwt`
//!   - X-Api-Key: `your_api_key`
//!   - X-Api-Secret: `your_api_secret`
//!   - Use either Authorization *or* X-Api-Key and X-Api-Secret to authenticate requests.
//! - Body: JSON specifying the request type (`type`) and the parameters (`params`).
//! The request type matches the name of the the request struct definition,
//! and the params match the fields of the request struct.
//!
//! For example, this is an example body for [read::GetDeployment]:
//! ```
//! {
//!   "type": "GetDeployment",
//!   "params": {
//!     "deployment": "66113df3abe32960b87018dd"
//!   }
//! }
//! ```
//!
//! The request's parent module (eg. [read], [mod@write]) determines the http path which
//! must be used for the requests. For example, requests under [read] are made using http path `/read`.
//!
//! ## Modules
//!
//! - [auth]: Requests relating to loggins in / obtaining authentication tokens.
//! - [read]: Read only requests which retrieve data from Monitor.
//! - [execute]: Run actions on monitor resources, eg [execute::RunBuild].
//! - [write]: Requests which alter data, like create / update / delete resources.
//!

pub mod auth;
pub mod execute;
pub mod read;
pub mod write;