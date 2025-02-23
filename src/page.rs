use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(bound = "for<'de2> T: Deserialize<'de2>")]
pub(crate) struct Page<T>
where
    for<'de2> T: Deserialize<'de2>,
{
    pub count: i32,
    pub all: Vec<i32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}
