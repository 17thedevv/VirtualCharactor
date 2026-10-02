use vc_runtime::tools::{KnowledgeLearner, WebSearchTool};

#[test]
fn test_web_search_trigger_detection() {
    assert_eq!(
        WebSearchTool::should_search("em có biết mck k"),
        Some("rapper MCK".to_string())
    );
    assert_eq!(
        WebSearchTool::should_search("thời tiết Hà Nội hôm nay"),
        Some("thời tiết hà nội".to_string())
    );
    assert_eq!(
        WebSearchTool::should_search("search Sơn Tùng M-TP"),
        Some("Sơn Tùng M-TP".to_string())
    );
    assert_eq!(WebSearchTool::should_search("chào em buổi sáng nha"), None);
}

#[test]
fn test_knowledge_learner_detection() {
    let t1 = KnowledgeLearner::detect_teaching("Dạy cho em nè: MCK là rapper nổi tiếng Việt Nam");
    assert!(t1.is_some());
    let fact1 = t1.unwrap();
    assert_eq!(fact1.fact, "MCK là rapper nổi tiếng Việt Nam");
    assert!(!fact1.is_user_preference);

    let t2 = KnowledgeLearner::detect_teaching("mình thích nghe nhạc lofi chill mỗi tối");
    assert!(t2.is_some());
    let fact2 = t2.unwrap();
    assert!(fact2.is_user_preference);

    let t3 = KnowledgeLearner::detect_teaching("Tại Vì Sao là bài hit của MCK");
    assert!(t3.is_some());

    let t4 = KnowledgeLearner::detect_teaching("hôm nay thời tiết đẹp quá nhỉ");
    assert!(t4.is_none());
}
