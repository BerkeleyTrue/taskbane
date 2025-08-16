async function initRegister(e) {
  e.preventDefault();
  const username = document.getElementById('username').value;
  if (!username) {
    alert('Please enter a username');
    return;
  }
  const creds = await fetch('/auth/register', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      username,
    }),
  })
    .then((res) => res.json())
    .then((credOptions) => credOptions.publicKey)
    .then(x => (console.log('credOptions: ', x), x))
    .then((publicKey) => ({
      ...publicKey,
      challenge: new TextEncoder().encode(publicKey.challenge).buffer,
      user: {
        ...publicKey.user,
        id: new TextEncoder().encode(publicKey.user.id).buffer,
      },
    }))
    .then((publicKey) => {
      return navigator.credentials.create({
        publicKey,
      });
    })
    .catch(err => {
      console.error('Error during registration:', err);
      alert('Registration failed. Please try again.');
    });

  console.log('creds: ', creds);
}
