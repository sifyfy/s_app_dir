// The MIT License (MIT)
//
// Copyright (c) 2016 Siphilia
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! # s_app_dir
//!
//! ## Usage
//!
//! Cargo.toml:
//!
//! ```toml
//! [package]
//! ...
//!
//! [dependencies]
//! s_app_dir = "*" # or semantic versioning
//! ```
//!
//! main.rc:
//!
//! ```rust
//! extern crate s_app_dir;
//!
//! use s_app_dir::{AppDir, XdgDir};
//!
//! fn main() {
//!     let app_dir = AppDir::new("foo-bar-app");
//!     println!("{:?}", app_dir.xdg_dir(XdgDir::Config));
//! }
//! ```

#![cfg_attr(any(feature="clippy", feature="sorty"), feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#![cfg_attr(feature="sorty", plugin(sorty))]
#![cfg_attr(feature="sorty", warn(unsorted_declarations))]

use std::env;
use std::fmt::{Display, Error, Formatter};
use std::path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XdgDir {
    Data,
    Config,
    Cache,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppDir {
    app_name: String,
}

impl AppDir {
    pub fn new(app_name: &str) -> AppDir {
        AppDir { app_name: app_name.to_string() }
    }

    #[cfg(unix)]
    fn xdg_dir_with_fallback<P>(&self, key: &str, fallback: P) -> Option<path::PathBuf>
        where P: AsRef<path::Path>
    {
        result_to_option(env::var(key))
            .map(|dir| path::PathBuf::new().join(&dir))
            .or(env::home_dir().map(|p| p.join(fallback)))
    }

    #[cfg(windows)]
    fn xdg_dir_with_fallback<P>(&self, key: &str, _: P) -> Option<path::PathBuf>
        where P: AsRef<path::Path>
    {
        result_to_option(env::var(key))
            .map(|dir| path::PathBuf::new().join(&dir))
            .or(result_to_option(env::var("APPDATA")).map(|dir| path::PathBuf::new().join(&dir)))
    }

    pub fn xdg_dir(&self, xdg: XdgDir) -> Option<path::PathBuf> {
        let xdg_dir = match xdg {
            XdgDir::Data => self.xdg_dir_with_fallback("XDG_DATA_HOME", ".local/share"),
            XdgDir::Config => self.xdg_dir_with_fallback("XDG_CONFIG_HOME", ".config"),
            XdgDir::Cache => self.xdg_dir_with_fallback("XDG_CACHE_HOME", ".cache"),
        };
        xdg_dir.map(|base| path::PathBuf::new().join(&base).join(&self.app_name))
    }

    #[cfg(unix)]
    pub fn user_data_dir(&self) -> Option<path::PathBuf> {
        env::home_dir().map(|p| p.join(".".to_string() + &self.app_name))
    }

    #[cfg(windows)]
    pub fn user_data_dir(&self) -> Option<path::PathBuf> {
        result_to_option(env::var("APPDATA"))
            .map(|v| path::PathBuf::new().join(v).join(&self.app_name))
    }

    pub fn temp_dir(&self) -> path::PathBuf {
        env::temp_dir().join(&self.app_name)
    }
}

impl Display for AppDir {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.app_name.fmt(f)
    }
}

fn result_to_option<T, E>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    static APP_NAME: &'static str = "s_app_dir";

    /// Return `None` or `$HOME/.local/share/app_name` based `std::env::home_dir()` if `XDG_DATA_HOME` is empty.
    #[cfg(unix)]
    #[test]
    fn default_data_home() {
        env::remove_var("XDG_DATA_HOME");
        let expect = env::home_dir().map(|p| p.join(".local/share").join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Data);
        assert_eq!(expect, value);
    }

    /// Return `None` or `%APPDATA%` on Windows if `XDG_DATA_HOME` is empty.
    #[cfg(windows)]
    #[test]
    fn default_data_home() {
        env::remove_var("XDG_DATA_HOME");
        let expect = ::result_to_option(env::var("APPDATA"))
                         .map(|dir| PathBuf::new().join(&dir).join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Data);
        assert_eq!(expect, value);
    }

    /// Return `$XDG_DATA_HOME/app_name` if `XDG_DATA_HOME` is set.
    #[test]
    fn env_data_home() {
        let xdg_data_home = PathBuf::new().join("/home/s_app_dir/.path/to/xdg_data_home");
        env::set_var("XDG_DATA_HOME", &xdg_data_home);

        let expect = Some(xdg_data_home.join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Data);
        assert_eq!(expect, value);
    }

    /// Return `None` or `$HOME/.config/app_name` based `std::env:home_dir()` if `XDG_CONFIG_HOME` is empty.
    #[cfg(unix)]
    #[test]
    fn default_config_home() {
        env::remove_var("XDG_CONFIG_HOME");
        let expect = env::home_dir().map(|p| p.join(".config").join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Config);
        assert_eq!(expect, value);
    }

    #[cfg(windows)]
    #[test]
    fn default_config_home() {
        env::remove_var("XDG_CONFIG_HOME");
        let expect = ::result_to_option(env::var("APPDATA"))
                         .map(|dir| PathBuf::new().join(&dir).join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Config);
        assert_eq!(expect, value);
    }

    /// Return `$XDG_CONFIG_HOME/app_name` if `XDG_CONFIG_HOME` is set.
    #[test]
    fn env_config_home() {
        let xdg_config_home = PathBuf::new().join("/home/s_app_dir/.path/to/xdg_config_home");
        env::set_var("XDG_CONFIG_HOME", &xdg_config_home);

        let expect = Some(xdg_config_home.join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Config);
        assert_eq!(expect, value);
    }

    /// Return `None` or `$HOME/.cache/app_name` based `std::env::home_dir()` if `XDG_CACHE_HOME` is empty.
    #[cfg(unix)]
    #[test]
    fn default_cache_home() {
        env::remove_var("XDG_CACHE_HOME");
        let expect = env::home_dir().map(|p| p.join(".cache").join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Cache);
        assert_eq!(expect, value);
    }

    #[cfg(windows)]
    #[test]
    fn default_cache_home() {
        env::remove_var("XDG_CACHE_HOME");
        let expect = ::result_to_option(env::var("APPDATA"))
                         .map(|dir| PathBuf::new().join(dir).join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Cache);
        assert_eq!(expect, value);
    }

    /// Return `$XDG_CACHE_HOME` if `XDG_CACHE_HOME` is set.
    #[test]
    fn env_cache_home() {
        let xdg_cache_home = PathBuf::new().join("/home/s_app_dir/.path/to/xdg_cache_home");
        env::set_var("XDG_CACHE_HOME", &xdg_cache_home);

        let expect = Some(xdg_cache_home.join(APP_NAME));
        let value = ::AppDir::new(APP_NAME).xdg_dir(::XdgDir::Cache);
        assert_eq!(expect, value);
    }

    /// Return `None` or `$HOME/.app_name`.
    #[cfg(unix)]
    #[test]
    fn user_data_dir() {
        let value = ::AppDir::new(APP_NAME).user_data_dir();
        let expect = env::home_dir().map(|p| p.join(".".to_string() + APP_NAME));
        assert_eq!(expect, value);
    }

    #[cfg(windows)]
    #[test]
    fn user_data_dir() {
        let value = ::AppDir::new(APP_NAME).user_data_dir();
        let expect = ::result_to_option(env::var("APPDATA"))
                         .map(|dir| PathBuf::new().join(dir).join(APP_NAME));
        assert_eq!(expect, value);
    }

    /// Return path based `std::env::temp_dir()`.
    #[test]
    fn temp_dir() {
        let value = ::AppDir::new(APP_NAME).temp_dir();
        let expect = env::temp_dir().join(APP_NAME);
        assert_eq!(expect, value);
    }
}
