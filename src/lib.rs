#![allow(dead_code)]

pub mod cli;
pub mod arg;
pub mod seqalin;
pub mod command;
pub mod pkgid;
mod cfgfile;
mod version;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}