use std::ops::{Deref, DerefMut};

use smol::lock::{Mutex, MutexGuard};

use crate::schemas::*;

pub static DB: Mutex<Database> = Mutex::new(Database {
    pets: Vec::new(),
    users: Vec::new(),
    tags: Vec::new(),
    orders: Vec::new(),
    categories: Vec::new(),
});

pub struct Database {
    pets: Vec<Pet>,
    users: Vec<User>,
    tags: Vec<Tag>,
    orders: Vec<Order>,
    categories: Vec<Category>,
}

impl PetstoreDB for MutexGuard<'_, Database> {}

pub trait PetstoreDB: DerefMut<Target = Database> {
    fn pets(&self) -> &Vec<Pet> {
        &self.pets
    }
    fn users(&self) -> &Vec<User> {
        &self.users
    }
    fn orders(&self) -> &Vec<Order> {
        &self.orders
    }
    fn categories(&self) -> &Vec<Category> {
        &self.categories
    }
    fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    fn get_pet(&self, id: i64) -> Option<&Pet> {
        self.pets.iter().find(|item| match item.id {
            Some(item_id) => item_id == id,
            None => false,
        })
    }

    fn get_pet_mut(&mut self, id: i64) -> Option<&mut Pet> {
        self.pets.iter_mut().find(|item| match item.id {
            Some(item_id) => item_id == id,
            None => false,
        })
    }

    fn pet_id(&self) -> i64 {
        for id in 0..i64::MAX {
            if self.get_pet(id).is_some() {
                continue;
            }
            return id;
        }
        unreachable!()
    }

    fn add_pet(&mut self, pet: Pet) -> Option<&Pet> {
        let id = pet.id.unwrap_or_else(|| self.pet_id());
        if self.get_pet(id).is_some() {
            return None;
        };
        self.pets.push(pet);
        self.pets.last()
    }
}
