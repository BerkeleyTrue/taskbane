async function initRegister() {
  const res = await fetch('/auth/register', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      username: 'exampleUser',
    }),
  });

  console.log("res", res);
}
