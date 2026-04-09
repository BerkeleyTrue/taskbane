var toUint = (s) => Base64.toUint8Array(s);
var fromUint = (s) => Base64.fromUint8Array(new Uint8Array(s), true);

async function addPasskey(e) {
  e.preventDefault();
  const creds = await fetch('/auth/register-sec-passkey', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
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
    .then((x) => (console.log('credOptions: ', x), x))
    .then((publicKey) => ({
      ...publicKey,
      challenge: toUint(publicKey.challenge),
      user: {
        ...publicKey.user,
        id: toUint(publicKey.user.id),
      },
      excludeCredentials: publicKey.excludeCredentials?.map((listItem) => ({
        ...listItem,
        id: toUint(listItem.id),
      })),
    }))
    .then((publicKey) => {
      return navigator.credentials.create({
        publicKey,
      });
    })
    .then((cred) => {
      return fetch('/auth/validate-sec-passkey', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
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
      });
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
      const hxRedirect = res.headers.get('hx-redirect');
      if (hxRedirect) {
        htmx.ajax('GET', hxRedirect, { target: 'body', swap: 'outerHTML' });
        return;
      }
      return res;
    })
    .catch((err) => {
      console.error('Error adding passkey:', err);
    });

  console.log('response: ', creds);
}
