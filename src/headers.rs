// TODO incomplete file

use crate::HttpMethod;

pub struct RequestHeaders {
	pub a_im: String,
	pub accept: Vec<MimeType>,
	pub accept_charset: Vec<Charset>,
	pub accept_datetime: chrono::DateTime<chrono::Utc>,
	pub accept_encoding: Vec<Encoding>,
	pub accept_language: Vec<Locale>,
	pub access_control_request_method: HttpMethod,
	pub access_control_request_headers: Vec<String>,
	pub authorization: Authorization,
	pub cache_control: CacheControl,
	pub connection: Connection,
	pub content_encoding: ContentEncoding,
	pub content_length: u64,
	pub content_md5: String,
	pub content_type: MimeType,
	pub cookie: Cookie,
	pub date: chrono::DateTime<chrono::Utc>,
	pub expect: String,
	pub forwarded: ForwardedHeaderValue,
	pub from: String,
	pub host: String,
	pub http2_settings: 
	pub if_match: 
	pub if_modified_since: 
	pub if_none_match: 
	pub if_range: 
	pub if_unmodified_since: 
	pub max_forwards: 
	pub origin: 
	pub pragma: 
	pub prefer: 
	pub proxy_authorization: 
	pub range: 
	pub referer: 
	pub te: 
	pub trailer: 
	pub transfer_encoding: 
	pub user_agent: 
	pub upgrade: 
	pub via: 
	pub warning: 
	pub upgrade_insecure_requests: 
	pub x_requested_with: 
	pub dnt: 
	pub x_forwarded_for: 
	pub x_forwarded_host: 
	pub x_forwarded_proto: 
	pub front_end_https: 
	pub x_http_method_override: 
	pub x_att_deviceid: 
	pub x_wap_profile: 
	pub proxy_connection: 
	pub x_uidh: 
	pub x_csrf_token: 
	pub x_request_id: 
	pub x_correlation_id: 
	pub save_data: 
}

pub enum Authorization {
	Basic {
		username: String,
		password: String,
	},
	Bearer {
		token: String,
	},
	Unknown(String),
	None,
}

pub struct Headers {
	pub accept: String,
	pub accept_charset: String,
	pub accept_encoding: Encoding,
	pub accept_language: String,
	pub accept_patch: Vec<String>,
	pub accept_post: Vec<String>,
	pub accept_ranges: Vec<String>,
	pub access_control_allow_credentials: bool,
	pub access_control_allow_headers: Vec<String>,
	pub access_control_allow_methods: Vec<HttpMethod>,
	pub access_control_allow_origin: Vec<String>,
	pub access_control_expose_headers: Vec<String>,
	pub access_control_max_age: u64,
	pub access_control_request_headers: Vec<String>,
	pub access_control_request_method: Vec<HttpMethod>,
	pub age: u64,
	pub allow: Vec<HttpMethod>,
	pub alt_svc: Vec<String>,
	pub authorization: String,
	pub cache_control: String,
	pub clear_site_data: Vec<String>,
	pub connection: ConnectionDirective,
	pub content_disposition: ContentDisposition,
	pub content_encoding: ContentEncoding,
	pub content_language: Vec<String>,
	pub content_length: u64,
	pub content_location: String,
	pub content_range: String,
	pub content_security_policy: String,
	pub content_security_policy_report_only: String,
	pub content_type: String,
	pub cookie: String,
	pub cross_origin_embedder_policy: String,
	pub cross_origin_opener_policy: String,
	pub cross_origin_resource_policy: String,
	pub dnt: String,
	pub dpr: String,
	pub date: String,
	pub device_memory: String,
	pub digest: String,
	pub etag: String,
	pub early_data: String,
	pub expect: String,
	pub expect_ct: String,
	pub expires: String,
	pub feature_policy: String,
	pub forwarded: String,
	pub from: String,
	pub host: String,
	pub if_match: String,
	pub if_modified_since: String,
	pub if_none_match: String,
	pub if_range: String,
	pub if_unmodified_since: String,
	pub index: String,
	pub keep_alive: String,
	pub large_allocation: String,
	pub last_modified: String,
	pub link: String,
	pub location: String,
	pub nel: String,
	pub origin: String,
	pub pragma: String,
	pub proxy_authenticate: String,
	pub proxy_authorization: String,
	pub public_key_pins: String,
	pub public_key_pins_report_only: String,
	pub range: String,
	pub referer: String,
	pub referrer_policy: String,
	pub retry_after: String,
	pub save_data: String,
	pub sec_fetch_dest: String,
	pub sec_fetch_mode: String,
	pub sec_fetch_site: String,
	pub sec_fetch_user: String,
	pub sec_websocket_accept: String,
	pub server: String,
	pub server_timing: String,
	pub set_cookie: String,
	pub sourcemap: String,
	pub strict_transport_security: String,
	pub te: String,
	pub timing_allow_origin: String,
	pub tk: String,
	pub trailer: String,
	pub transfer_encoding: String,
	pub upgrade: String,
	pub upgrade_insecure_requests: String,
	pub user_agent: String,
	pub vary: String,
	pub via: String,
	pub www_authenticate: String,
	pub want_digest: String,
	pub warning: String,
	pub x_content_type_options: String,
	pub x_dns_prefetch_control: String,
	pub x_forwarded_for: String,
	pub x_forwarded_host: String,
	pub x_forwarded_proto: String,
	pub x_frame_options: String,
	pub x_xss_protection: String,
}

pub enum Encoding {
	Gzip,
	Compress,
	Deflate,
	Identity,
	Brotli,
	Any,
	Other(String)
}

pub enum ContentDisposition {
	Inline,
	Attachment {
		file_name: String
	},
	FormData {
		name: String,
		file_name: Option<String>
	}
}

pub enum ConnectionDirective {
	Close,
	Headers(Vec<String>)
}

pub struct ContentRangeDirective {
	pub unit: ContentRangeUnit,
	pub range_start: u64,
	pub range_end: u64,
	pub size: u64,
}
