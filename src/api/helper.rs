use crate::{
    api::dto::content::{
        FriendsHops, InviteCreateOptions, RelationshipQuery, SubsetQuery, SubsetQueryOptions,
        TypedMessage,
    },
    feed::Message,
    rpc::{ArgType, Body, BodyType, RequestNo, RpcType, RpcWriter},
};
use async_std::io::Write;

use super::{dto, error::Result};

const MAX_RPC_BODY_LEN: usize = 65536;

#[derive(Debug)]
pub enum ApiMethod {
    BlobsCreateWants,
    BlobsGet,
    CreateFeedStream,
    CreateHistoryStream,
    FriendsBlocks,
    FriendsHops,
    FriendsIsFollowing,
    FriendsIsBlocking,
    Get,
    GetSubset,
    InviteCreate,
    InviteUse,
    Latest,
    NamesGet,
    NamesGetImageFor,
    NamesGetSignifier,
    PrivatePublish,
    Publish,
    TanglesThread,
    WhoAmI,
}

impl ApiMethod {
    pub fn selector(&self) -> &'static [&'static str] {
        use ApiMethod::*;
        match self {
            BlobsCreateWants => &["blobs", "createWants"],
            BlobsGet => &["blobs", "get"],
            CreateFeedStream => &["createFeedStream"],
            CreateHistoryStream => &["createHistoryStream"],
            FriendsBlocks => &["friends", "blocks"],
            FriendsHops => &["friends", "hops"],
            FriendsIsBlocking => &["friends", "isBlocking"],
            FriendsIsFollowing => &["friends", "isFollowing"],
            Get => &["get"],
            GetSubset => &["partialReplication", "getSubset"],
            InviteCreate => &["invite", "create"],
            InviteUse => &["invite", "use"],
            Latest => &["latest"],
            NamesGet => &["names", "get"],
            NamesGetImageFor => &["names", "getImageFor"],
            NamesGetSignifier => &["names", "getSignifier"],
            PrivatePublish => &["private", "publish"],
            Publish => &["publish"],
            TanglesThread => &["tangles", "thread"],
            WhoAmI => &["whoami"],
        }
    }
    pub fn from_selector(s: &[&str]) -> Option<Self> {
        use ApiMethod::*;
        match s {
            ["blobs", "createWants"] => Some(BlobsCreateWants),
            ["blobs", "get"] => Some(BlobsGet),
            ["createFeedStream"] => Some(CreateFeedStream),
            ["createHistoryStream"] => Some(CreateHistoryStream),
            ["friends", "blocks"] => Some(FriendsBlocks),
            ["friends", "hops"] => Some(FriendsHops),
            ["friends", "isBlocking"] => Some(FriendsIsBlocking),
            ["friends", "isFollowing"] => Some(FriendsIsFollowing),
            ["get"] => Some(Get),
            ["partialReplication", "getSubset"] => Some(GetSubset),
            ["invite", "create"] => Some(InviteCreate),
            ["invite", "use"] => Some(InviteUse),
            ["latest"] => Some(Latest),
            ["names", "get"] => Some(NamesGet),
            ["names", "getImageFor"] => Some(NamesGetImageFor),
            ["names", "getSignifier"] => Some(NamesGetSignifier),
            ["private", "publish"] => Some(PrivatePublish),
            ["publish"] => Some(Publish),
            ["tangles", "thread"] => Some(TanglesThread),
            ["whoami"] => Some(WhoAmI),
            _ => None,
        }
    }
    pub fn from_rpc_body(body: &Body) -> Option<Self> {
        let selector = body.name.iter().map(|v| v.as_str()).collect::<Vec<_>>();
        Self::from_selector(&selector)
    }
}

pub struct ApiCaller<W: Write + Unpin> {
    rpc: RpcWriter<W>,
}

impl<W: Write + Unpin> ApiCaller<W> {
    pub fn new(rpc: RpcWriter<W>) -> Self {
        Self { rpc }
    }

    pub fn rpc(&mut self) -> &mut RpcWriter<W> {
        &mut self.rpc
    }

    /// Send blob create wants.
    pub async fn blob_create_wants_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::BlobsCreateWants.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["blobs","get"] request.
    pub async fn blobs_get_req_send(&mut self, args: &dto::BlobsGetIn) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::BlobsGet.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send blob response.
    pub async fn blobs_get_res_send<D: AsRef<[u8]>>(
        &mut self,
        req_no: RequestNo,
        data: D,
    ) -> Result<()> {
        let mut offset = 0;
        let data = data.as_ref();
        while offset < data.len() {
            let limit = std::cmp::min(data.len(), offset + MAX_RPC_BODY_LEN);
            self.rpc
                .send_response(
                    req_no,
                    RpcType::Source,
                    BodyType::Binary,
                    &data[offset..limit],
                )
                .await?;
            offset += MAX_RPC_BODY_LEN;
        }
        self.rpc.send_stream_eof(req_no).await?;
        Ok(())
    }

    /// Send ["createFeedStream"] request.
    pub async fn create_feed_stream_req_send<'a>(
        &mut self,
        args: &dto::CreateStreamIn<u64>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::CreateFeedStream.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["createHistoryStream"] request.
    pub async fn create_history_stream_req_send(
        &mut self,
        args: &dto::CreateHistoryStreamIn,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::CreateHistoryStream.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send feed response.
    pub async fn feed_res_send(&mut self, req_no: RequestNo, feed: &str) -> Result<()> {
        self.rpc
            .send_response(req_no, RpcType::Source, BodyType::JSON, feed.as_bytes())
            .await?;
        Ok(())
    }

    /// Send ["friends", "blocks"] request.
    pub async fn friends_blocks_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::FriendsBlocks.selector(),
                RpcType::Source,
                ArgType::Object,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["friends", "hops"] request.
    pub async fn friends_hops_req_send(&mut self, args: FriendsHops) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::FriendsHops.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["friends", "isBlocking"] request.
    pub async fn friends_is_blocking_req_send(
        &mut self,
        args: RelationshipQuery,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::FriendsIsBlocking.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["friends", "isFollowing"] request.
    pub async fn friends_is_following_req_send(
        &mut self,
        args: RelationshipQuery,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::FriendsIsFollowing.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["get"] request.
    pub async fn get_req_send(&mut self, msg_id: &str) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::Get.selector(),
                RpcType::Async,
                ArgType::Array,
                &msg_id,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["get"] response.
    pub async fn get_res_send(&mut self, req_no: RequestNo, msg: &Message) -> Result<()> {
        self.rpc
            .send_response(
                req_no,
                RpcType::Async,
                BodyType::JSON,
                msg.to_string().as_bytes(),
            )
            .await?;
        Ok(())
    }

    /// Send ["partialReplication", "getSubset"] request.
    pub async fn getsubset_req_send(
        &mut self,
        query: SubsetQuery,
        opts: Option<SubsetQueryOptions>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::GetSubset.selector(),
                RpcType::Source,
                ArgType::Tuple,
                &query,
                &opts,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["invite", "create"] request.
    pub async fn invite_create_req_send(&mut self, uses: u16) -> Result<RequestNo> {
        let args = InviteCreateOptions { uses };
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::InviteCreate.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["invite", "use"] request.
    pub async fn invite_use_req_send(&mut self, invite_code: &str) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::InviteUse.selector(),
                RpcType::Async,
                ArgType::Array,
                &invite_code,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["latest"] request.
    pub async fn latest_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::Latest.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["names", "get"] request.
    pub async fn names_get_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::NamesGet.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["names", "getImageFor"] request.
    pub async fn names_get_image_for_req_send(&mut self, feed_id: &str) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::NamesGetImageFor.selector(),
                RpcType::Async,
                ArgType::Array,
                &feed_id,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["names", "getSignifier"] request.
    pub async fn names_get_signifier_req_send(&mut self, feed_id: &str) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::NamesGetSignifier.selector(),
                RpcType::Async,
                ArgType::Array,
                &feed_id,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["publish"] request.
    pub async fn publish_req_send(&mut self, msg: TypedMessage) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::Publish.selector(),
                RpcType::Async,
                ArgType::Array,
                &msg,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["private", "publish"] request.
    pub async fn private_publish_req_send(
        &mut self,
        msg: TypedMessage,
        recipients: Vec<String>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::PrivatePublish.selector(),
                RpcType::Async,
                ArgType::Tuple,
                &msg,
                &Some(recipients),
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["publish"] response.
    pub async fn publish_res_send(&mut self, req_no: RequestNo, msg_ref: String) -> Result<()> {
        Ok(self
            .rpc
            .send_response(req_no, RpcType::Async, BodyType::JSON, msg_ref.as_bytes())
            .await?)
    }

    /// Send ["tangles", "thread"] request.
    pub async fn tangles_thread_req_send(
        &mut self,
        args: &dto::TanglesThread,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::TanglesThread.selector(),
                RpcType::Source,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["whoami"] request.
    pub async fn whoami_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::WhoAmI.selector(),
                RpcType::Async,
                ArgType::Array,
                &args,
                &None::<()>,
            )
            .await?;
        Ok(req_no)
    }

    /// Send ["whoami"] response.
    pub async fn whoami_res_send(&mut self, req_no: RequestNo, id: String) -> Result<()> {
        let body = serde_json::to_string(&dto::WhoAmIOut { id })?;
        Ok(self
            .rpc
            .send_response(req_no, RpcType::Async, BodyType::JSON, body.as_bytes())
            .await?)
    }
}
