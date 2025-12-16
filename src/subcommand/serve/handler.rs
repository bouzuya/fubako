mod get;
mod get_image;
mod get_page_by_title;
mod get_root_or_list_pages;
mod get_script_index;
mod get_style_index;
mod list;

pub use self::get::handle as get;
pub use self::get_image::handle as get_image;
pub use self::get_page_by_title::handle as get_page_by_title;
pub use self::get_root_or_list_pages::handle as get_root_or_list_pages;
pub use self::get_script_index::handle as get_script_index;
pub use self::get_style_index::handle as get_style_index;
pub use self::list::handle as list;
