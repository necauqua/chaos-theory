
/* explicitly global to be easier to call from wasm */
window._game_error = (text) => {
    let p = document.createElement('p');
    p.style.color = 'red';
    p.style.whiteSpace = 'pre-line';
    p.style.margin = '1rem';
    p.innerText = 'Error:\n' + text;
    document.body.textContent = '';
    document.body.appendChild(p);
}

import('./index.js')
  .catch(e => {
      _game_error(e.toString())
      console.error(e);
  });
