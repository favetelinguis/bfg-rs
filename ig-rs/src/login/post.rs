imports!();
use crate::client::PostQueryBuilder;

new_type!(
    Session
    RefreshToken
);

from!(
    @PostQueryBuilder
        -> Session = "session"
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
