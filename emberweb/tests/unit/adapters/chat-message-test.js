import { module, test } from 'qunit';
import { setupTest } from 'ember-qunit';

module('Unit | Adapter | chat message', function(hooks) {
  setupTest(hooks);

  // Replace this with your real tests.
  test('it exists', function(assert) {
    let adapter = this.owner.lookup('adapter:chat_message');
    assert.ok(adapter);
  });
});
