async function initRegister() {
  const username = document.getElementById('username').value;
  if (!username) {
    alert('Please enter a username');
    return;
  }
  const res = await fetch('/auth/register', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      username,
    }),
  });

  console.log("res", res);
}
