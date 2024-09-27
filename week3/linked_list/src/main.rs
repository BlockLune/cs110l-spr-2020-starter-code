use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<String> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i.to_string());
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    let mut list_2: LinkedList<String> = LinkedList::new();
    for i in 1..11 {
        list_2.push_front(i.to_string());
    }
    assert_eq!(list, list_2);

    let list_3 = list_2.clone();
    assert_eq!(list_2, list_3);

    println!("---");

    // If you implement iterator trait:
    for val in &list {
        println!("{}", val);
    }

    println!("---");

    for val in list_2 {
        println!("{}", val);
    }
}
