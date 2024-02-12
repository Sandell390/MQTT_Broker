#[cfg(test)]
mod tests {
    use crate::{add_client_to_topic_list, models::topic::Topic, remove_client_from_topic_list};


    #[test]
    fn test_add_client_to_topic_list() {
        let mut topics = vec![
            Topic {
                topic_name: "topic1".to_string(),
                retained_msg: (String::new(), 0),
                client_ids: vec![("client1".to_string(), 0)],
            },
            Topic {
                topic_name: "topic2".to_string(),
                retained_msg: (String::new(), 0),
                client_ids: vec![("client2".to_string(), 0)],
            },
        ];

        // Test adding a client to an existing topic
        add_client_to_topic_list(&mut topics, "client3".to_string(), ("topic1".to_string(), 0));
        assert_eq!(topics[0].client_ids.len(), 2);
        assert_eq!(topics[0].client_ids[1], ("client3".to_string(), 0));

        // Test adding a client to a new topic
        add_client_to_topic_list(&mut topics, "client4".to_string(), ("topic3".to_string(), 0));
        assert_eq!(topics.len(), 3);
        assert_eq!(topics[2].topic_name, "topic3");
        assert_eq!(topics[2].client_ids[0], ("client4".to_string(), 0));
    }

    #[test]
    fn test_remove_client_from_topic_list() {
        let mut topics = vec![
            Topic {
                topic_name: "topic1".to_string(),
                retained_msg: (String::new(), 0),
                client_ids: vec![("client1".to_string(), 0), ("client2".to_string(), 0)],
            },
            Topic {
                topic_name: "topic2".to_string(),
                retained_msg: (String::new(), 0),
                client_ids: vec![("client3".to_string(), 0)],
            },
        ];

        // Test removing a client from an existing topic
        remove_client_from_topic_list(&mut topics, "client1".to_string(), ("topic1".to_string(), 0));
        assert_eq!(topics[0].client_ids.len(), 1);
        assert_eq!(topics[0].client_ids[0], ("client2".to_string(), 0));

        // Test removing a client from a non-existing topic
        remove_client_from_topic_list(&mut topics, "client4".to_string(), ("topic3".to_string(), 0));
        assert_eq!(topics.len(), 2); // No new topic should be created
    }
}