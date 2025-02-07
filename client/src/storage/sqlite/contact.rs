use sqlx::{Pool, Sqlite};

use crate::storage::ContactStore;

#[derive(Debug)]
pub struct SqliteContactStore {
    _database: Pool<Sqlite>,
}

impl SqliteContactStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self {
            _database: database,
        }
    }
}

impl ContactStore for SqliteContactStore {}

/*

   #[tokio::test]
   async fn store_and_load_contact() {
       // store_contact
       // load_contact
       let device = Device::new(connect().await);

       let contacts = vec![new_contact(), new_contact(), new_contact()];

       device.store_contact(&contacts[0]).await.unwrap();
       device.store_contact(&contacts[1]).await.unwrap();
       device.store_contact(&contacts[2]).await.unwrap();

       let retrived_contacts = device.load_contacts().await.unwrap();

       assert_eq!(contacts, retrived_contacts);
   }

   #[tokio::test]
   async fn insert_and_get_address_by_nickname() {
       // insert_address_for_nickname
       // get_address_by_nickname
       let device = Device::new(connect().await);

       let nicknames = vec!["Alice", "Bob", "Charlie"];

       let nickname_map = HashMap::from([
           (nicknames[0], new_service_id()),
           (nicknames[1], new_service_id()),
           (nicknames[2], new_service_id()),
       ]);

       device
           .insert_service_id_for_nickname(nicknames[0], &nickname_map[nicknames[0]])
           .await
           .unwrap();
       device
           .insert_service_id_for_nickname(nicknames[1], &nickname_map[nicknames[1]])
           .await
           .unwrap();
       device
           .insert_service_id_for_nickname(nicknames[2], &nickname_map[nicknames[2]])
           .await
           .unwrap();

       assert_eq!(
           device
               .get_service_id_by_nickname(nicknames[0])
               .await
               .unwrap(),
           nickname_map[nicknames[0]]
       );
       assert_eq!(
           device
               .get_service_id_by_nickname(nicknames[1])
               .await
               .unwrap(),
           nickname_map[nicknames[1]]
       );
       assert_eq!(
           device
               .get_service_id_by_nickname(nicknames[2])
               .await
               .unwrap(),
           nickname_map[nicknames[2]]
       );
   }
*/
