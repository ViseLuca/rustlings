Tried to manage transaction like an ATM. <br>

I tried to use the type system to ensure correctness and bring with me all the transaction in the
case where I have to check all the data to find a specific one.

I didn't set a transaction in dispute but in case I searched for the same transaction number
with a dispute opened and not resolved or chargedback. If the account has a chargeback, is locked and I won't accept any new operation on it.

I tried to cover every corner case with unit test, I don't have any other case in particular
 
Chargeback and Resolve I assumed that are final states, so even if I have a Chargeback on Dispute will
block the account and no other actions are allowed, if it is resolved is not allowed a charged back
