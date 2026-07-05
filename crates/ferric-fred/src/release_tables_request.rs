use crate::{Client, ReleaseElementId, ReleaseTable, Result};

/// A builder for a `release/tables` request, returned by
/// [`Client::release_tables`]. Fetches a release's table tree — the whole tree
/// by default, or the subtree rooted at one element via
/// [`element`](ReleaseTablesRequest::element). Finish with
/// [`send`](ReleaseTablesRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::ReleaseId;
/// let table = client.release_tables(ReleaseId::new(10)).send().await?;
/// println!("{} root elements", table.roots.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a ReleaseTablesRequest does nothing until you call `.send()`"]
pub struct ReleaseTablesRequest<'a> {
    client: &'a Client,
    release_id: u32,
    element_id: Option<ReleaseElementId>,
}

impl<'a> ReleaseTablesRequest<'a> {
    pub(crate) fn new(client: &'a Client, release_id: u32) -> Self {
        Self {
            client,
            release_id,
            element_id: None,
        }
    }

    /// Fetch only the subtree rooted at this element (`element_id`), instead of
    /// the release's whole table tree.
    pub fn element(mut self, element_id: ReleaseElementId) -> Self {
        self.element_id = Some(element_id);
        self
    }

    /// Run the request and return the (sub)tree.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<ReleaseTable> {
        self.client.execute_release_tables(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> =
            vec![("release_id", self.release_id.to_string())];
        if let Some(element_id) = self.element_id {
            params.push(("element_id", element_id.get().to_string()));
        }
        params
    }
}
