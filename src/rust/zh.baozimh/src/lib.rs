#![no_std]

use aidoku::{
	error::Result,
	prelude::*,
	std::{
		defaults::defaults_get,
		format,
		net::{HttpMethod, Request},
		print, String, StringRef, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};

mod filter;
mod helper;

const POPULAR_MANGA_SELECTOR: &str = "div.pure-g div.comics-card";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	print("get_manga_list");

	let base_url = defaults_get("mirror")?
		.as_string()
		.unwrap_or_else(|_| StringRef::from(""))
		.read();

	let url = get_manga_list_url(filters, page)?;
	let html = Request::new(url, HttpMethod::Get).html()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	print(format(format_args!(
		"{}",
		html.select(POPULAR_MANGA_SELECTOR).array().count()
	)));

	for item in html.select(POPULAR_MANGA_SELECTOR).array() {
		match item.as_node() {
			Ok(n) => {
				let poster = n.select("> a.comics-card__poster");
				let info = n.select("> a.comics-card__info");

				let href = poster.attr("href").read();
				let cover = poster.select("> amp-img").attr("src").read();
				let id = href.replace("/comic/", "");

				let title = info.select("> .comics-card__title").text().read();
				let author = info.select("> .tags").text().read();

				let mut categories = Vec::new();

				for t in poster.select("span.tab").array() {
					match t.as_node() {
						Ok(n) => {
							categories.push(String::from(n.text().read().trim()));
						}
						Err(_) => continue,
					};
				}

				manga_arr.push(Manga {
					id,
					cover,
					title,
					author,
					artist: String::from(""),
					description: String::from(""),
					url: format(format_args!("{}{}", base_url, href)),
					categories,
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Scroll,
				});
			}
			Err(_) => continue,
		}
	}

	let has_more = !manga_arr.is_empty();

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	todo!()
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	todo!()
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	todo!()
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	todo!()
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}

fn get_manga_list_url(filters: Vec<Filter>, page: i32) -> Result<String> {
	let mut title = String::new();
	let mut category = String::new();
	let mut region = String::new();
	let mut status = String::new();
	let mut start = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"状态" => {
						status = String::from(filter::STATUS[index]);
					}
					"地区" => {
						region = String::from(filter::REGION[index]);
					}
					"分类" => {
						category = String::from(filter::CATEGORY[index]);
					}
					"标题开头" => {
						start = String::from(filter::START[index]);
					}
					_ => continue,
				};
			}
			_ => continue,
		}
	}

	print(&title);
	print(&category);
	print(&region);
	print(&status);
	print(&start);

	let mut url = defaults_get("mirror")?
		.as_string()
		.unwrap_or_else(|_| StringRef::from(""))
		.read();

	url.push_str("/classify?page=");
	url.push_str(&helper::i32_to_string(page));
	url.push('&');

	if !category.is_empty() {
		url.push_str("type=");
		url.push_str(&category);
		url.push('&');
	}

	if !region.is_empty() {
		url.push_str("region=");
		url.push_str(&region);
		url.push('&');
	}

	if !status.is_empty() {
		url.push_str("state=");
		url.push_str(&status);
		url.push('&');
	}

	if !start.is_empty() {
		url.push_str("filter=");
		url.push_str(&start);
	}

	print("url:");
	print(&url);

	Ok(url)
}
