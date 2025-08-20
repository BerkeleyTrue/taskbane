var toUint = (s) => Base64.toUint8Array(s);
var fromUint = (s) => Base64.fromUint8Array(new Uint8Array(s), true);

async function login(e) {
  e.preventDefault();
  const username = document.getElementById('username').value;
  if (!username) {
    alert('Please enter a username');
    return;
  }
  const creds = await fetch('/auth/login', {
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
      allowCredentials: publicKey.allowCredentials?.map((listItem) => ({
        ...listItem,
        id: toUint(listItem.id),
      }))
    }))
    .then((publicKey) => {
      return navigator.credentials.get({
        publicKey,
      });
    })
    .then((assertion) => {
      return fetch("/auth/validate-login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          id: assertion.id,
          rawId: fromUint(assertion.rawId),
          type: assertion.type,
          response: {
            authenticatorData: fromUint(assertion.response.authenticatorData),
            clientDataJSON: fromUint(assertion.response.clientDataJSON),
            signature: fromUint(assertion.response.signature),
            userHandle: fromUint(assertion.response.userHandle),
          },
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
