var toUint = (s) => Base64.toUint8Array(s);
var fromUint = (s) => Base64.fromUint8Array(new Uint8Array(s), true);

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
    .then((res) => {
      if (!res.ok) {
        return res.json().then((err) => {
          throw new Error(err.message || 'Failed to fetch registration options');
        });
      }
      return res;
    })
    .then((res) => res.json())
    .then((credOptions) => credOptions.publicKey)
    .then(x => (console.log('credOptions: ', x), x))
    .then((publicKey) => ({
      ...publicKey,
      challenge: toUint(publicKey.challenge),
      user: {
        ...publicKey.user,
        id: toUint(publicKey.user.id),
      },
    }))
    .then((publicKey) => {
      return navigator.credentials.create({
        publicKey,
      });
    })
    .then((cred) => {
      return fetch("/auth/validate-registration", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          id: cred.id,
          rawId: fromUint(cred.rawId),
          response: {
            attestationObject: fromUint(cred.response.attestationObject),
            clientDataJSON: fromUint(cred.response.clientDataJSON),
          },
          type: cred.type,
        }),
        redirect: 'follow', // doesn't seem to work
      })
    })
    .then((res) => {
      if (!res.ok) {
        return res.json().then((err) => {
          throw new Error(err.message || 'Failed to validate registration');
        });
      }
      return res;
    })
    .then((res) => {
      if (res.redirected) {
        window.location.href = res.url;
      }
      return res;
    })
    .catch(err => {
      console.error('Error during registration:', err);
    });

  console.log('response: ', creds);
}
