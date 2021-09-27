/**
 * @fileoverview gRPC-Web generated client stub for lambda_view
 * @enhanceable
 * @public
 */

// GENERATED CODE -- DO NOT EDIT!


/* eslint-disable */
// @ts-nocheck



const grpc = {};
grpc.web = require('grpc-web');

const proto = {};
proto.lambda_view = require('./lambda_view_pb.js');

/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.lambda_view.LambdaViewClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.lambda_view.LambdaViewPromiseClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.lambda_view.HelloRequest,
 *   !proto.lambda_view.HelloReply>}
 */
const methodDescriptor_LambdaView_SayHello = new grpc.web.MethodDescriptor(
  '/lambda_view.LambdaView/SayHello',
  grpc.web.MethodType.UNARY,
  proto.lambda_view.HelloRequest,
  proto.lambda_view.HelloReply,
  /**
   * @param {!proto.lambda_view.HelloRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.lambda_view.HelloReply.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.lambda_view.HelloRequest,
 *   !proto.lambda_view.HelloReply>}
 */
const methodInfo_LambdaView_SayHello = new grpc.web.AbstractClientBase.MethodInfo(
  proto.lambda_view.HelloReply,
  /**
   * @param {!proto.lambda_view.HelloRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.lambda_view.HelloReply.deserializeBinary
);


/**
 * @param {!proto.lambda_view.HelloRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.lambda_view.HelloReply)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.lambda_view.HelloReply>|undefined}
 *     The XHR Node Readable Stream
 */
proto.lambda_view.LambdaViewClient.prototype.sayHello =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/lambda_view.LambdaView/SayHello',
      request,
      metadata || {},
      methodDescriptor_LambdaView_SayHello,
      callback);
};


/**
 * @param {!proto.lambda_view.HelloRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.lambda_view.HelloReply>}
 *     Promise that resolves to the response
 */
proto.lambda_view.LambdaViewPromiseClient.prototype.sayHello =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/lambda_view.LambdaView/SayHello',
      request,
      metadata || {},
      methodDescriptor_LambdaView_SayHello);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.lambda_view.ParamsRequest,
 *   !proto.lambda_view.ParamsResponse>}
 */
const methodDescriptor_LambdaView_StreamParams = new grpc.web.MethodDescriptor(
  '/lambda_view.LambdaView/StreamParams',
  grpc.web.MethodType.SERVER_STREAMING,
  proto.lambda_view.ParamsRequest,
  proto.lambda_view.ParamsResponse,
  /**
   * @param {!proto.lambda_view.ParamsRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.lambda_view.ParamsResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.lambda_view.ParamsRequest,
 *   !proto.lambda_view.ParamsResponse>}
 */
const methodInfo_LambdaView_StreamParams = new grpc.web.AbstractClientBase.MethodInfo(
  proto.lambda_view.ParamsResponse,
  /**
   * @param {!proto.lambda_view.ParamsRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.lambda_view.ParamsResponse.deserializeBinary
);


/**
 * @param {!proto.lambda_view.ParamsRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.lambda_view.ParamsResponse>}
 *     The XHR Node Readable Stream
 */
proto.lambda_view.LambdaViewClient.prototype.streamParams =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/lambda_view.LambdaView/StreamParams',
      request,
      metadata || {},
      methodDescriptor_LambdaView_StreamParams);
};


/**
 * @param {!proto.lambda_view.ParamsRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.lambda_view.ParamsResponse>}
 *     The XHR Node Readable Stream
 */
proto.lambda_view.LambdaViewPromiseClient.prototype.streamParams =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/lambda_view.LambdaView/StreamParams',
      request,
      metadata || {},
      methodDescriptor_LambdaView_StreamParams);
};


module.exports = proto.lambda_view;

