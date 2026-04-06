use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct WebSearchResult {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct WebFetchResult {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    #[test]
    fn web_search_result_extracts_fields() {
        let title = "title text".to_string();
        let url = "https://en.wikipedia.org/wiki/Drain_Gang".to_string();

        let v = json!(
          {
            "type": "web_search_result",
            "title": title,
            "url": url,
            "encrypted_content": "blahblah",
            "page_age": "6 hours ago",
          }
        );

        let res = from_value::<WebSearchResult>(v).unwrap();
        assert_eq!(res, WebSearchResult { title, url });
    }

    #[test]
    fn web_fetch_result_extracts_fields() {
        let url = "https://www.wyatt.wtf".to_string();

        let v = json!(
          {
            "type": "web_fetch_result",
            "url": url,
            "retrieved_at": "2026-04-06T04:11:06.948315",
            "content": {
              "type": "document",
              "source": {
                "type": "text",
                "media_type": "text/plain",
                "data": "wyatt.wtf\n\n# under construction rn\n\ncheck back later :)"
              },
              "title": "under construction rn"
            }
          }
        );

        let res = from_value::<WebFetchResult>(v).unwrap();
        assert_eq!(res, WebFetchResult { url });
    }
}
