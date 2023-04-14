use crate::prelude::{
    FlatIdPool,
    IdPool,
};

#[test]
fn test_flat_id_pool() {
    let mut pool = FlatIdPool::new(0_u16);
    let zero = pool.request().unwrap();
    let one = pool.request().unwrap();

    assert_eq!(zero, 0);
    assert_eq!(one, 1);

    pool.return_id(zero);
    pool.return_id(one);

    assert_eq!(pool.request().unwrap(), one);
    assert_eq!(pool.request().unwrap(), zero);
}
