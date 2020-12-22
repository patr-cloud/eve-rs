// TODO incomplete file

use crate::HttpMethod;

pub struct RequestHeaders {
	
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
	pub content_Security-Policy-Report-Only
	pub content_Type
	Cookie
	Cross-Origin-Embedder-Policy
	Cross-Origin-Opener-Policy
	Cross-Origin-Resource-Policy
	DNT
	DPR
	Date
	Device-Memory
	Digest
	ETag
	Early-Data
	Expect
	Expect-CT
	Expires
	Feature-Policy
	Forwarded
	From
	Host
	If-Match
	If-Modified-Since
	If-None-Match
	If-Range
	If-Unmodified-Since
	Index
	Keep-Alive
	Large-Allocation
	Last-Modified
	Link
	Location
	NEL
	Origin
	Pragma
	Proxy-Authenticate
	Proxy-Authorization
	Public-Key-Pins
	Public-Key-Pins-Report-Only
	Range
	Referer
	Referrer-Policy
	Retry-After
	Save-Data
	Sec-Fetch-Dest
	Sec-Fetch-Mode
	Sec-Fetch-Site
	Sec-Fetch-User
	Sec-WebSocket-Accept
	Server
	Server-Timing
	Set-Cookie
	SourceMap
	Strict-Transport-Security
	TE
	Timing-Allow-Origin
	Tk
	Trailer
	Transfer-Encoding
	Upgrade
	Upgrade-Insecure-Requests
	User-Agent
	Vary
	Via
	WWW-Authenticate
	Want-Digest
	Warning
	X-Content-Type-Options
	X-DNS-Prefetch-Control
	X-Forwarded-For
	X-Forwarded-Host
	X-Forwarded-Proto
	X-Frame-Options
	X-XSS-Protection
}

pub enum Encoding {
	Gzip,
	Compress,
	Deflate,
	Identity,
	Brotli,
	Any
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

(AcceptEncoding, ACCEPT_ENCODING, "accept-encoding");
(AcceptLanguage, ACCEPT_LANGUAGE, "accept-language");
(AcceptRanges, ACCEPT_RANGES, "accept-ranges");
(AccessControlAllowCredentials, ACCESS_CONTROL_ALLOW_CREDENTIALS, "access-control-allow-credentials");
(AccessControlAllowHeaders, ACCESS_CONTROL_ALLOW_HEADERS, "access-control-allow-headers");
(AccessControlAllowMethods, ACCESS_CONTROL_ALLOW_METHODS, "access-control-allow-methods");
(AccessControlAllowOrigin, ACCESS_CONTROL_ALLOW_ORIGIN, "access-control-allow-origin");
(AccessControlExposeHeaders, ACCESS_CONTROL_EXPOSE_HEADERS, "access-control-expose-headers");
(AccessControlMaxAge, ACCESS_CONTROL_MAX_AGE, "access-control-max-age");
(AccessControlRequestHeaders, ACCESS_CONTROL_REQUEST_HEADERS, "access-control-request-headers");
(AccessControlRequestMethod, ACCESS_CONTROL_REQUEST_METHOD, "access-control-request-method");
(Age, AGE, "age");
(Allow, ALLOW, "allow");
(AltSvc, ALT_SVC, "alt-svc");
(Authorization, AUTHORIZATION, "authorization");
(CacheControl, CACHE_CONTROL, "cache-control");
(Connection, CONNECTION, "connection");
(ContentDisposition, CONTENT_DISPOSITION, "content-disposition");
(ContentEncoding, CONTENT_ENCODING, "content-encoding");
(ContentLanguage, CONTENT_LANGUAGE, "content-language");
(ContentLength, CONTENT_LENGTH, "content-length");
(ContentLocation, CONTENT_LOCATION, "content-location");
(ContentRange, CONTENT_RANGE, "content-range");
(ContentSecurityPolicy, CONTENT_SECURITY_POLICY, "content-security-policy");
(ContentSecurityPolicyReportOnly, CONTENT_SECURITY_POLICY_REPORT_ONLY, "content-security-policy-report-only");
(ContentType, CONTENT_TYPE, "content-type");
(Cookie, COOKIE, "cookie");
(Dnt, DNT, "dnt");
(Date, DATE, "date");
(Etag, ETAG, "etag");
(Expect, EXPECT, "expect");
(Expires, EXPIRES, "expires");
(Forwarded, FORWARDED, "forwarded");
(From, FROM, "from");
(Host, HOST, "host");
(IfMatch, IF_MATCH, "if-match");
(IfModifiedSince, IF_MODIFIED_SINCE, "if-modified-since");
(IfNoneMatch, IF_NONE_MATCH, "if-none-match");
(IfRange, IF_RANGE, "if-range");
(IfUnmodifiedSince, IF_UNMODIFIED_SINCE, "if-unmodified-since");
(LastModified, LAST_MODIFIED, "last-modified");
(Link, LINK, "link");
(Location, LOCATION, "location");
(MaxForwards, MAX_FORWARDS, "max-forwards");
(Origin, ORIGIN, "origin");
(Pragma, PRAGMA, "pragma");
(ProxyAuthenticate, PROXY_AUTHENTICATE, "proxy-authenticate");
(ProxyAuthorization, PROXY_AUTHORIZATION, "proxy-authorization");
(PublicKeyPins, PUBLIC_KEY_PINS, "public-key-pins");
(PublicKeyPinsReportOnly, PUBLIC_KEY_PINS_REPORT_ONLY, "public-key-pins-report-only");
(Range, RANGE, "range");
(Referer, REFERER, "referer");
(ReferrerPolicy, REFERRER_POLICY, "referrer-policy");
(Refresh, REFRESH, "refresh");
(RetryAfter, RETRY_AFTER, "retry-after");
(SecWebSocketAccept, SEC_WEBSOCKET_ACCEPT, "sec-websocket-accept");
(SecWebSocketExtensions, SEC_WEBSOCKET_EXTENSIONS, "sec-websocket-extensions");
(SecWebSocketKey, SEC_WEBSOCKET_KEY, "sec-websocket-key");
(SecWebSocketProtocol, SEC_WEBSOCKET_PROTOCOL, "sec-websocket-protocol");
(SecWebSocketVersion, SEC_WEBSOCKET_VERSION, "sec-websocket-version");
(Server, SERVER, "server");
(SetCookie, SET_COOKIE, "set-cookie");
(StrictTransportSecurity, STRICT_TRANSPORT_SECURITY, "strict-transport-security");
(Te, TE, "te");
(Trailer, TRAILER, "trailer");
(TransferEncoding, TRANSFER_ENCODING, "transfer-encoding");
(UserAgent, USER_AGENT, "user-agent");
(Upgrade, UPGRADE, "upgrade");
(UpgradeInsecureRequests, UPGRADE_INSECURE_REQUESTS, "upgrade-insecure-requests");
(Vary, VARY, "vary");
(Via, VIA, "via");
(Warning, WARNING, "warning");
(WwwAuthenticate, WWW_AUTHENTICATE, "www-authenticate");
(XContentTypeOptions, X_CONTENT_TYPE_OPTIONS, "x-content-type-options");
(XDnsPrefetchControl, X_DNS_PREFETCH_CONTROL, "x-dns-prefetch-control");
(XFrameOptions, X_FRAME_OPTIONS, "x-frame-options");
(XXssProtection, X_XSS_PROTECTION, "x-xss-protection");