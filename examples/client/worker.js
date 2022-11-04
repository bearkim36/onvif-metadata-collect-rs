const { WorkerData, parentPort } = require('worker_threads')
const sleep = require('thread-sleep');

const addon = require('./index.node');
addon.add_metadata_connect((val)=> {
  console.log(val);
});