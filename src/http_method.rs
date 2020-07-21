#[derive(Eq, PartialEq, Hash, Clone)]
pub enum HttpMethod {
	Get,
	Post,
	Put,
	Delete,
	Head,
	Options,
	Connect,
	Patch,
	Trace,
	Use,
}
