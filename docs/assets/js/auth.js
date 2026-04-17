// Password: 20kbc2026
// To change: echo -n "newpassword" | shasum -a 256, then replace HASH below.
(function () {
  var HASH = "235773a3b1d2087c7296c69e9847482e728ae4215beff65511d90339f077f64e";
  var KEY = "20kbc_auth";

  function sha256(str) {
    return crypto.subtle.digest("SHA-256", new TextEncoder().encode(str)).then(function (buf) {
      return Array.from(new Uint8Array(buf)).map(function (b) { return b.toString(16).padStart(2, "0"); }).join("");
    });
  }

  function unlock() {
    sessionStorage.setItem(KEY, "1");
    document.documentElement.classList.add("authenticated");
    var g = document.getElementById("auth-gate");
    if (g) g.remove();
  }

  // Already authed this session
  if (sessionStorage.getItem(KEY) === "1") { unlock(); return; }

  // Build gate UI
  var gate = document.createElement("div");
  gate.id = "auth-gate";
  gate.innerHTML =
    '<div class="box">' +
    '<h2>20,000 BC</h2>' +
    '<p>Enter the password to view this documentation.</p>' +
    '<input type="password" id="auth-pw" placeholder="Password" autofocus />' +
    '<button id="auth-btn">Enter</button>' +
    '<div class="err" id="auth-err"></div>' +
    '</div>';
  document.body.insertBefore(gate, document.body.firstChild);

  document.getElementById("auth-btn").addEventListener("click", function () {
    var pw = document.getElementById("auth-pw").value;
    sha256(pw).then(function (h) {
      if (h === HASH) { unlock(); }
      else {
        document.getElementById("auth-err").textContent = "Incorrect password.";
        document.getElementById("auth-pw").value = "";
        document.getElementById("auth-pw").focus();
      }
    });
  });

  document.getElementById("auth-pw").addEventListener("keydown", function (e) {
    if (e.key === "Enter") document.getElementById("auth-btn").click();
  });
})();
