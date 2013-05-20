var connect = require('connect');
var http = require('http');
var rootPath = __dirname + '/../';

var app = connect()
  .use(connect.static(rootPath))
  .use(connect.directory(rootPath));

http.createServer(app).listen(3000);