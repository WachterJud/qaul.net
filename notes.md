# libqaul api scopes

* contacts
  * modify(userauth, contact-id, lambda)
  * get(userauth, contact-id)
  * query(userauth, contact query)
  * all(userauth)
* messages
  * send(userauth, mode, id_type, into service, into tagset, payload (vec u8))
  * subscribe(userauth, into service, into tagset)
  * query(userauth, into service, msgquery)
* services
  * register(service name, callback(service event))
  * unregister(name)
  * save(userauth, into service, metadata map, into tagset)
  * delete(userauth, into service, into key)
  * query(userauth, into service, into tagset)
* users
  * list()
  * list_remote() (???)
  * is_authenticated(userauth)
  * create(password)
  * delete(userauth)
  * change_pw(userauth, new password)
  * login(user id, password)
  * logout(userauth)
  * get(user id)
  * update(userauth, user update)
  
