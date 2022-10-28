mod mongo;
mod redis;

pub use self::{
    mongo::{FreezersStore, ProductsStore},
    redis::{ImageStore, RoleStore},
};
