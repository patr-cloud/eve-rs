use std::{fmt::Debug, time::Instant};

use chrono::Local;
use colored::Colorize;

use crate::Context;

pub struct LoggerMiddleware<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	log_format: String,
	should_skip: fn(&TContext) -> bool,
	measurer: Option<Instant>,
}

impl<TContext> LoggerMiddleware<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	pub fn create(
		log_format: &str,
		should_skip: fn(&TContext) -> bool,
	) -> LoggerMiddleware<TContext> {
		let log_format = match log_format {
			"tiny" => ":method :url :status :res[content-length] - :response-time",
			"short" => ":remote-addr :remote-user :method :url HTTP/:http-version :status :res[content-length] - :response-time",
			"dev" => ":method :url :status :response-time - :res[content-length]",
			"common" => ":remote-addr - :remote-user [:date[clf]] \":method :url HTTP/:http-version\" :status :res[content-length]",
			"combined" => ":remote-addr - :remote-user [:date[clf]] \":method :url HTTP/:http-version\" :status :res[content-length] \":referrer\" \":user-agent\"",
			_ => log_format,
		}
		.to_string();
		LoggerMiddleware {
			log_format,
			should_skip,
			measurer: None,
		}
	}

	pub fn begin_measuring(&mut self) {
		self.measurer = Some(Instant::now());
	}

	pub fn complete_measuring(&mut self, context: &TContext) -> Option<String> {
		if self.measurer.is_none() {
			log::warn!("invalid state! ensure you call logger::begin_measuring before calling logger::complete_measuring");
			return None;
		}
		let elapsed_time =
			Instant::now().duration_since(self.measurer.unwrap());

		if (self.should_skip)(context) {
			return None;
		}
		let reqs = self
			.log_format
			.match_indices(":req[")
			.filter_map(|(index, _)| {
				let header_end_index =
					self.log_format[index..].chars().position(|c| c == ']')?;
				let header_name =
					&self.log_format[(index + 5)..header_end_index];
				Some((
					header_name.to_string(),
					context.get_header(header_name)?,
				))
			})
			.collect::<Vec<(String, String)>>();

		let ress = self
			.log_format
			.match_indices(":res[")
			.filter_map(|(index, _)| {
				let header_end_index =
					self.log_format[index..].chars().position(|c| c == ']')?;
				let header_name =
					&self.log_format[(index + 5)..(index + header_end_index)];
				Some((
					header_name.to_string(),
					context.get_response().get_header(header_name)?,
				))
			})
			.collect::<Vec<(String, String)>>();

		let dates = vec![
			("clf", Local::now().format("%d/%b/%Y:%H:%M:%S %z")),
			("iso", Local::now().format("%Y-%m-%dT%H:%M:%S.%3fZ")),
			("web", Local::now().format("%a, %d %b %Y %H:%M:%S %Z")),
		];

		let mut log_value = self
			.log_format
			.replace(
				":http-version",
				&format!(
					"{}.{}",
					context.get_request().get_version_major(),
					context.get_request().get_version_minor()
				),
			)
			.replace(":method", &context.get_method().to_string())
			.replace(
				":referrer",
				&context.get_header("Referer").unwrap_or_else(|| {
					context.get_header("Referrer").unwrap_or_default()
				}),
			)
			.replace(":remote-addr", &context.get_ip().to_string())
			.replace(
				":response-time",
				&if elapsed_time.as_millis() > 0 {
					format!("{} ms", elapsed_time.as_millis())
				} else {
					format!("{} μs", elapsed_time.as_micros())
				},
			)
			.replace(
				":status",
				&match context.get_response().get_status() {
					100..=199 => {
						format!("{}", context.get_response().get_status())
							.normal()
					}
					200..=299 => {
						format!("{}", context.get_response().get_status())
							.green()
					}
					300..=399 => {
						format!("{}", context.get_response().get_status())
							.cyan()
					}
					400..=499 => {
						format!("{}", context.get_response().get_status())
							.yellow()
					}
					500..=599 => {
						format!("{}", context.get_response().get_status()).red()
					}
					_ => format!("{}", context.get_response().get_status())
						.purple(),
				},
			)
			.replace(":url", &context.get_path())
			.replace(
				":user-agent",
				&context.get_header("User-Agent").unwrap_or_default(),
			)
			.replace(
				":content-length",
				&context
					.get_request()
					.get_header("content-length")
					.unwrap_or_else(|| "-".to_string()),
			)
			.replace(":date", ":date[web]");

		for (key, value) in reqs {
			log_value = log_value.replace(&format!(":req[{}]", key), &value);
		}
		for (key, value) in ress {
			log_value = log_value.replace(&format!(":res[{}]", key), &value);
		}
		for (key, value) in dates {
			log_value = log_value
				.replace(&format!(":date[{}]", key), &value.to_string());
		}

		Some(log_value)
	}
}

pub fn default<TContext>() -> LoggerMiddleware<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	LoggerMiddleware::create("dev", |_| false)
}

pub fn with_format<TContext>(format: &str) -> LoggerMiddleware<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	LoggerMiddleware::create(format, |_| false)
}

pub fn with_format_and_skippable<TContext>(
	format: &str,
	should_skip: fn(&TContext) -> bool,
) -> LoggerMiddleware<TContext>
where
	TContext: Context + Debug + Send + Sync,
{
	LoggerMiddleware::create(format, should_skip)
}
