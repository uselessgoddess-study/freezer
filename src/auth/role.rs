

use num_enum::{IntoPrimitive, TryFromPrimitive};
use redis::{ErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};

use tap::Conv;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive)]
#[repr(usize)]
pub enum Role {
    Admin,
    Moder,
    User,
}

impl FromRedisValue for Role {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let repr: usize = usize::from_redis_value(v)?;
        Role::try_from(repr).map_err(|_| {
            RedisError::from((
                ErrorKind::TypeError,
                "Response was of incompatible value",
                format!(
                    "allow range {}..{} (response was {:?})",
                    Role::Admin as usize,
                    Role::User as usize, // todo: calc automatically
                    v
                ),
            ))
        })
    }
}

impl ToRedisArgs for Role {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        ((*self).conv::<usize>()).write_redis_args(out)
    }
}
