use ratatui::widgets::{BarChart, BarGroup, Bar};
fn test() {
    let g1 = BarGroup::default().bars(&[Bar::default().value(1)]);
    let groups = vec![g1];
    let b = BarChart::default();
    // How to assign groups to b?
}
