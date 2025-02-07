export function hasShtmlAttribute(node) {
  if (node.tagName.toLowerCase() === "img") {
    // replace "srv:" by server url
    const attributes = node.attributes;
    for (let i = 0; i < attributes.length; i++) {
        if (attributes[i].name === 'src') {
            const src = attributes[i].value;
            if (src.startsWith('srv:')) {
                attributes[i].value = src.replace('srv:', window.IMMT_SERVER_URL);
            }
        }
    }
  }
  //if (node.tagName.toLowerCase() === "section") {return true}
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
      if (attributes[i].name.startsWith('data-shtml-')) {
          return true;
      }
  }
  return false;
}

window.IMMT_SERVER_URL = "";

export function setServerUrl(url) {
  window.IMMT_SERVER_URL = url;
  set_server_url(url);
}

/*
function getDataShtmlAttributes(node) {
  const result = [];
  const attributes = node.attributes;
  for (let i = 0; i < attributes.length; i++) {
      if (attributes[i].name.startsWith('data-shtml-')) {
          result.push(attributes[i].name);
      }
  }
  return result;
}
  */