imports!();
use crate::client::PostQueryBuilder;

new_type!(
    Session
    RefreshToken
);

from!(
    @PostQueryBuilder
        -> Session = "session|3"
    @Session
        -> RefreshToken = "refresh-token"
);

impl_macro!(
    @Session
        |=> refresh_token -> RefreshToken
        |
);

exec!(Session);
exec!(RefreshToken);
