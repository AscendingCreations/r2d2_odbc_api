extern crate odbc_api;
extern crate r2d2;
extern crate r2d2_odbc_api;

use r2d2::Pool;
use r2d2_odbc_api::ODBCConnectionManager;

use std::sync::mpsc;
use std::thread;

#[test]
fn test_smoke() {
    let manager = ODBCConnectionManager::new("DSN=PostgreSQL");
    let pool = Pool::builder().max_size(2).build(manager).unwrap();

    let (s1, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();

    let pool1 = pool.clone();
    let t1 = thread::spawn(move || {
        let conn = pool1.get().unwrap();
        s1.send(()).unwrap();
        r2.recv().unwrap();
        drop(conn);
    });

    let pool2 = pool.clone();
    let t2 = thread::spawn(move || {
        let conn = pool2.get().unwrap();
        s2.send(()).unwrap();
        r1.recv().unwrap();
        drop(conn);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    pool.get().unwrap();
}
