#![no_std]

mod helper;
use aidoku::{
	error::Result,
	prelude::*,
	std::{
		format, json,
		net::{HttpMethod, Request},
		print, ArrayRef, ObjectRef, String, Vec,
	},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};

const BASE_URL: &str = "https://rouman5.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	print("get_manga_list");

	let mut query = String::new();
	let mut tag = String::new();
	let mut status = 0;
	let mut sort = 0;

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"標籤" => {
						let options = filter.object.get("options").as_array()?;
						tag = options.get(index).as_string()?.read();
					}
					"狀態" => {
						status = index;
					}
					_ => continue,
				};
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int()? as usize;
				sort = index;
			}
			_ => continue,
		}
	}

	print(&format(format_args!("query: {}", &query)));
	print(&format(format_args!("tag: {}", &tag)));
	print(&format(format_args!("status: {}", status)));
	print(&format(format_args!("sort: {}", sort)));

	if query.is_empty() && tag == "全部" && status == 0 && sort == 0 {
		get_home_page_manga_list()
	} else if !query.is_empty() {
		get_search_manga_list(query, page)
	} else {
		get_filter_manga_list(ListFilter { tag, status, sort }, page)
	}
}

#[get_manga_listing]
fn get_manga_listing(_: Listing, _: i32) -> Result<MangaPageResult> {
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
fn modify_image_request(_: Request) {}

#[handle_url]
fn handle_url(_: String) -> Result<DeepLink> {
	todo!()
}

fn book_to_manga_with_cover_key(book: ObjectRef, cover_key: &str) -> Result<Manga> {
	let id = book.get("id").as_string()?.read();
	let mut url = String::from(BASE_URL);
	url.push_str(&format(format_args!("/books/{}", id)));

	let mut categories: Vec<String> = Vec::new();

	for t in book.get("tags").as_array()? {
		let tag = t.as_string()?.read();
		categories.push(tag);
	}

	Ok(Manga {
		id,
		cover: book.get(cover_key).as_string()?.read(),
		title: book.get("name").as_string()?.read(),
		author: book.get("author").as_string()?.read(),
		artist: String::from(""),
		description: book.get("description").as_string()?.read(),
		url,
		categories,
		status: match book.get("continued").as_bool()? {
			true => MangaStatus::Ongoing,
			false => MangaStatus::Completed,
		},
		nsfw: MangaContentRating::Nsfw,
		viewer: MangaViewer::Scroll,
	})
}

fn book_to_manga(book: ObjectRef) -> Result<Manga> {
	book_to_manga_with_cover_key(book, "coverUrl")
}

fn get_home_page_manga_list() -> Result<MangaPageResult> {
	let url = format(format_args!("{}/home", BASE_URL));
	print(&url);

	let html = Request::new(&url, HttpMethod::Get).html()?;

	let json_text = html.select("#__NEXT_DATA__").html().read();

	// print(&json_text);

	let json = json::parse(json_text)?.as_object()?;
	let props = json.get("props").as_object()?;
	let data = props.get("pageProps").as_object()?;

	let headline = data.get("headline").as_object()?;
	let best = data.get("best").as_array()?;
	let hottest = data.get("hottest").as_array()?;
	let daily = data.get("daily").as_array()?;
	let recent_updated_books = data.get("recentUpdatedBooks").as_array()?;
	let ended_books = data.get("endedBooks").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	manga_arr.push(book_to_manga_with_cover_key(headline, "coverUrlSquare")?);

	for b in best {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}
	for b in hottest {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}
	for b in daily {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}
	for b in recent_updated_books {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}
	for b in ended_books {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: false,
	})
}

fn get_filter_manga_list(filter: ListFilter, page: i32) -> Result<MangaPageResult> {
	let mut url = format(format_args!("{}/books?", BASE_URL));

	if filter.tag != "全部" {
		url.push_str(&format(format_args!(
			"tag={}&",
			helper::urlencode(filter.tag)
		)));
	}

	if filter.status > 0 {
		url.push_str(&format(format_args!(
			"continued={}&",
			match filter.status {
				1 => "true",
				_ => "false",
			}
		)));
	}

	if filter.sort > 0 {
		url.push_str("sort=rating&")
	}

	url.push_str(&format(format_args!("page={}", page - 1)));

	print(&url);

	let html = Request::new(&url, HttpMethod::Get).html()?;

	let json_text = html.select("#__NEXT_DATA__").html().read();
	let json = json::parse(json_text)?.as_object()?;
	let props = json.get("props").as_object()?;
	let data = props.get("pageProps").as_object()?;

	let books = data.get("books").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	for b in books {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}

	let has_more = !manga_arr.is_empty();
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more,
	})
}

fn get_search_manga_list(query: String, page: i32) -> Result<MangaPageResult> {
	let mut url = format(format_args!("{}/search?", BASE_URL));

	url.push_str(&format(format_args!("term={}&", helper::urlencode(query))));
	url.push_str(&format(format_args!("page={}", page - 1)));

	print(&url);

	let html = Request::new(&url, HttpMethod::Get).html()?;

	let json_text = html.select("#__NEXT_DATA__").html().read();
	let json = json::parse(json_text)?.as_object()?;
	let props = json.get("props").as_object()?;
	let data = props.get("pageProps").as_object()?;

	let books = data.get("books").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	for b in books {
		manga_arr.push(book_to_manga(b.as_object()?)?);
	}

	let has_more = !manga_arr.is_empty();
	Ok(MangaPageResult {
		manga: manga_arr,
		has_more,
	})
}
struct ListFilter {
	tag: String,
	status: usize,
	sort: usize,
}
