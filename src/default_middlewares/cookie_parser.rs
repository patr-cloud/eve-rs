use crate::{Context, Cookie, CookieOptions, DefaultMiddleware, SameSite};
use std::fmt::Debug;

pub fn parser<TContext>(context: &mut TContext)
where
	TContext: Context + Debug + Send + Sync,
{
	let header = context.get_request().get_headers().get("Cookie");
	if header.is_none() {
		return;
	}
	let header = header.unwrap().clone();

	context
		.get_request_mut()
		.cookies
		.extend(header.into_iter().map(|header| {
			let mut options = CookieOptions::default();

			let mut pieces = header.split(';');

			let mut key_pair = pieces.next().unwrap().split('=');

			let key = key_pair.next().unwrap_or("").to_owned();
			let value = key_pair.next().unwrap_or("").to_owned();

			for option in pieces {
				let mut option_key_pair = option.split('=');

				if let Some(option_key) = option_key_pair.next() {
					match option_key.to_lowercase().trim() {
						"expires" => {
							options.expires = option_key_pair
								.next()
								.unwrap_or("0")
								.parse::<u64>()
								.unwrap_or(0)
						}
						"max-age" => {
							options.max_age = option_key_pair
								.next()
								.unwrap_or("0")
								.parse::<u64>()
								.unwrap_or(0)
						}
						"domain" => {
							options.domain = option_key_pair.next().unwrap_or("").to_owned()
						}
						"path" => options.path = option_key_pair.next().unwrap_or("").to_owned(),
						"secure" => options.secure = true,
						"httponly" => options.http_only = true,
						"samesite" => {
							if let Some(same_site_value) = option_key_pair.next() {
								match same_site_value.to_lowercase().as_ref() {
									"strict" => options.same_site = Some(SameSite::Strict),
									"lax" => options.same_site = Some(SameSite::Lax),
									_ => (),
								};
							}
						}
						_ => (),
					};
				}
			}

			Cookie {
				key,
				value,
				options,
			}
		}));
}

pub fn default_parser<TData>() -> DefaultMiddleware<TData>
where
	TData: Default + Clone + Send + Sync,
{
	DefaultMiddleware::new(|mut context, next| {
		Box::pin(async move {
			parser(&mut context);

			next(context).await
		})
	})
}
