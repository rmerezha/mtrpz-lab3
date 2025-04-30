use std::error::Error;
use postgres_types::{FromSql, IsNull, to_sql_checked, ToSql, Type};
use postgres_types::private::BytesMut;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema, Default, Clone, PartialEq)]
pub enum UserTokenStatus {
    #[default]
    ACTIVE,
    INACTIVE,
}

impl FromSql<'_> for UserTokenStatus {
    fn from_sql(
        _sql_type: &Type,
        value: &[u8],
    ) -> Result<Self, Box<dyn Error + Sync + Send>> {
        match value {
            b"ACTIVE" => Ok(UserTokenStatus::ACTIVE),
            b"INACTIVE" => Ok(UserTokenStatus::INACTIVE),
            _ => Ok(UserTokenStatus::ACTIVE),
        }
    }

    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "varchar"
    }
}

impl ToSql for UserTokenStatus {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {

        format!("{:?}", self).to_sql(ty, out)
    }

    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "varchar"
    }

    to_sql_checked!();
}
