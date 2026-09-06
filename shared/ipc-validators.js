/* Generated from Rust JSON Schema. No runtime eval. */
//#region \0rolldown/runtime.js
var __commonJSMin = (cb, mod) => () => (mod || (cb((mod = { exports: {} }).exports, mod), cb = null), mod.exports);
//#endregion
//#region node_modules/ajv/dist/runtime/ucs2length.js
var require_ucs2length = /* @__PURE__ */ __commonJSMin(((exports) => {
	Object.defineProperty(exports, "__esModule", { value: true });
	function ucs2length(str) {
		const len = str.length;
		let length = 0;
		let pos = 0;
		let value;
		while (pos < len) {
			length++;
			value = str.charCodeAt(pos++);
			if (value >= 55296 && value <= 56319 && pos < len) {
				value = str.charCodeAt(pos);
				if ((value & 64512) === 56320) pos++;
			}
		}
		return length;
	}
	exports.default = ucs2length;
	ucs2length.code = "require(\"ajv/dist/runtime/ucs2length\").default";
}));
//#endregion
//#region shared/ipc-validators-input.cjs
var require_ipc_validators_input = /* @__PURE__ */ __commonJSMin(((exports) => {
	exports.Aggregate = validate37;
	function validate37(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate37.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId")) {
				validate37.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType")) {
					validate37.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.aggregateId !== void 0) {
					if (typeof data.aggregateId !== "string") {
						validate37.errors = [{
							instancePath: instancePath + "/aggregateId",
							schemaPath: "#/properties/aggregateId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateType !== void 0) {
						if (typeof data.aggregateType !== "string") {
							validate37.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
				}
			}
		} else {
			validate37.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate37.errors = vErrors;
		return true;
	}
	validate37.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.CommandEnvelope = validate38;
	var func1 = require_ucs2length().default;
	function validate38(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate38.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.command === void 0 && (missing0 = "command") || data.payload === void 0 && (missing0 = "payload")) {
				validate38.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "command" || key0 === "payload" || key0 === "requestId" || key0 === "schemaVersion")) {
					validate38.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.command !== void 0) {
					if (typeof data.command !== "string") {
						validate38.errors = [{
							instancePath: instancePath + "/command",
							schemaPath: "#/properties/command/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.requestId !== void 0) {
						let data1 = data.requestId;
						if (typeof data1 === "string") {
							if (func1(data1) > 128) {
								validate38.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate38.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate38.errors = [{
								instancePath: instancePath + "/requestId",
								schemaPath: "#/properties/requestId/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.schemaVersion !== void 0) {
							let data2 = data.schemaVersion;
							if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
								validate38.errors = [{
									instancePath: instancePath + "/schemaVersion",
									schemaPath: "#/properties/schemaVersion/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								}];
								return false;
							}
							if (1 !== data2) {
								validate38.errors = [{
									instancePath: instancePath + "/schemaVersion",
									schemaPath: "#/properties/schemaVersion/const",
									keyword: "const",
									params: { allowedValue: 1 },
									message: "must be equal to constant"
								}];
								return false;
							}
							if (typeof data2 == "number") {
								if (data2 < 0 || isNaN(data2)) {
									validate38.errors = [{
										instancePath: instancePath + "/schemaVersion",
										schemaPath: "#/properties/schemaVersion/minimum",
										keyword: "minimum",
										params: {
											comparison: ">=",
											limit: 0
										},
										message: "must be >= 0"
									}];
									return false;
								}
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate38.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate38.errors = vErrors;
		return true;
	}
	validate38.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.DomainEvent = validate39;
	function validate39(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate39.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.eventId === void 0 && (missing0 = "eventId") || data.eventType === void 0 && (missing0 = "eventType") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.occurredAt === void 0 && (missing0 = "occurredAt") || data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.sequence === void 0 && (missing0 = "sequence") || data.payload === void 0 && (missing0 = "payload")) {
				validate39.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "eventId" || key0 === "eventType" || key0 === "occurredAt" || key0 === "payload" || key0 === "schemaVersion" || key0 === "sequence")) {
					validate39.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.aggregateId !== void 0) {
					if (typeof data.aggregateId !== "string") {
						validate39.errors = [{
							instancePath: instancePath + "/aggregateId",
							schemaPath: "#/properties/aggregateId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateType !== void 0) {
						let data1 = data.aggregateType;
						if (typeof data1 !== "string") {
							validate39.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						if ("workspace" !== data1) {
							validate39.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/const",
								keyword: "const",
								params: { allowedValue: "workspace" },
								message: "must be equal to constant"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.eventId !== void 0) {
							if (typeof data.eventId !== "string") {
								validate39.errors = [{
									instancePath: instancePath + "/eventId",
									schemaPath: "#/properties/eventId/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.eventType !== void 0) {
								let data3 = data.eventType;
								if (typeof data3 !== "string") {
									validate39.errors = [{
										instancePath: instancePath + "/eventType",
										schemaPath: "#/properties/eventType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if ("workspace.opened" !== data3) {
									validate39.errors = [{
										instancePath: instancePath + "/eventType",
										schemaPath: "#/properties/eventType/const",
										keyword: "const",
										params: { allowedValue: "workspace.opened" },
										message: "must be equal to constant"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.occurredAt !== void 0) {
									if (typeof data.occurredAt !== "string") {
										validate39.errors = [{
											instancePath: instancePath + "/occurredAt",
											schemaPath: "#/properties/occurredAt/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.payload !== void 0) {
										let data5 = data.payload;
										if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
											let missing1;
											if (data5.workspaceId === void 0 && (missing1 = "workspaceId") || data5.name === void 0 && (missing1 = "name") || data5.baseCurrency === void 0 && (missing1 = "baseCurrency") || data5.path === void 0 && (missing1 = "path") || data5.createdAt === void 0 && (missing1 = "createdAt") || data5.lastOpenedAt === void 0 && (missing1 = "lastOpenedAt") || data5.storageSchemaVersion === void 0 && (missing1 = "storageSchemaVersion")) {
												validate39.errors = [{
													instancePath: instancePath + "/payload",
													schemaPath: "#/$defs/Workspace/required",
													keyword: "required",
													params: { missingProperty: missing1 },
													message: "must have required property '" + missing1 + "'"
												}];
												return false;
											} else {
												for (const key1 in data5) if (!(key1 === "baseCurrency" || key1 === "createdAt" || key1 === "lastOpenedAt" || key1 === "name" || key1 === "path" || key1 === "storageSchemaVersion" || key1 === "workspaceId")) {
													validate39.errors = [{
														instancePath: instancePath + "/payload",
														schemaPath: "#/$defs/Workspace/additionalProperties",
														keyword: "additionalProperties",
														params: { additionalProperty: key1 },
														message: "must NOT have additional properties"
													}];
													return false;
												}
												if (data5.baseCurrency !== void 0) {
													if (typeof data5.baseCurrency !== "string") {
														validate39.errors = [{
															instancePath: instancePath + "/payload/baseCurrency",
															schemaPath: "#/$defs/Workspace/properties/baseCurrency/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid2 = true;
												} else var valid2 = true;
												if (valid2) {
													if (data5.createdAt !== void 0) {
														if (typeof data5.createdAt !== "string") {
															validate39.errors = [{
																instancePath: instancePath + "/payload/createdAt",
																schemaPath: "#/$defs/Workspace/properties/createdAt/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid2 = true;
													} else var valid2 = true;
													if (valid2) {
														if (data5.lastOpenedAt !== void 0) {
															if (typeof data5.lastOpenedAt !== "string") {
																validate39.errors = [{
																	instancePath: instancePath + "/payload/lastOpenedAt",
																	schemaPath: "#/$defs/Workspace/properties/lastOpenedAt/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid2 = true;
														} else var valid2 = true;
														if (valid2) {
															if (data5.name !== void 0) {
																if (typeof data5.name !== "string") {
																	validate39.errors = [{
																		instancePath: instancePath + "/payload/name",
																		schemaPath: "#/$defs/Workspace/properties/name/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																var valid2 = true;
															} else var valid2 = true;
															if (valid2) {
																if (data5.path !== void 0) {
																	if (typeof data5.path !== "string") {
																		validate39.errors = [{
																			instancePath: instancePath + "/payload/path",
																			schemaPath: "#/$defs/Workspace/properties/path/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid2 = true;
																} else var valid2 = true;
																if (valid2) {
																	if (data5.storageSchemaVersion !== void 0) {
																		let data11 = data5.storageSchemaVersion;
																		if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11))) {
																			validate39.errors = [{
																				instancePath: instancePath + "/payload/storageSchemaVersion",
																				schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/type",
																				keyword: "type",
																				params: { type: "integer" },
																				message: "must be integer"
																			}];
																			return false;
																		}
																		if (1 !== data11) {
																			validate39.errors = [{
																				instancePath: instancePath + "/payload/storageSchemaVersion",
																				schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/const",
																				keyword: "const",
																				params: { allowedValue: 1 },
																				message: "must be equal to constant"
																			}];
																			return false;
																		}
																		if (typeof data11 == "number") {
																			if (data11 < 0 || isNaN(data11)) {
																				validate39.errors = [{
																					instancePath: instancePath + "/payload/storageSchemaVersion",
																					schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																					keyword: "minimum",
																					params: {
																						comparison: ">=",
																						limit: 0
																					},
																					message: "must be >= 0"
																				}];
																				return false;
																			}
																		}
																		var valid2 = true;
																	} else var valid2 = true;
																	if (valid2) {
																		if (data5.workspaceId !== void 0) {
																			if (typeof data5.workspaceId !== "string") {
																				validate39.errors = [{
																					instancePath: instancePath + "/payload/workspaceId",
																					schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																					keyword: "type",
																					params: { type: "string" },
																					message: "must be string"
																				}];
																				return false;
																			}
																			var valid2 = true;
																		} else var valid2 = true;
																	}
																}
															}
														}
													}
												}
											}
										} else {
											validate39.errors = [{
												instancePath: instancePath + "/payload",
												schemaPath: "#/$defs/Workspace/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.schemaVersion !== void 0) {
											let data13 = data.schemaVersion;
											if (!(typeof data13 == "number" && !(data13 % 1) && !isNaN(data13))) {
												validate39.errors = [{
													instancePath: instancePath + "/schemaVersion",
													schemaPath: "#/properties/schemaVersion/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												}];
												return false;
											}
											if (1 !== data13) {
												validate39.errors = [{
													instancePath: instancePath + "/schemaVersion",
													schemaPath: "#/properties/schemaVersion/const",
													keyword: "const",
													params: { allowedValue: 1 },
													message: "must be equal to constant"
												}];
												return false;
											}
											if (typeof data13 == "number") {
												if (data13 < 0 || isNaN(data13)) {
													validate39.errors = [{
														instancePath: instancePath + "/schemaVersion",
														schemaPath: "#/properties/schemaVersion/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													}];
													return false;
												}
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.sequence !== void 0) {
												let data14 = data.sequence;
												if (!(typeof data14 == "number" && !(data14 % 1) && !isNaN(data14))) {
													validate39.errors = [{
														instancePath: instancePath + "/sequence",
														schemaPath: "#/properties/sequence/type",
														keyword: "type",
														params: { type: "integer" },
														message: "must be integer"
													}];
													return false;
												}
												if (typeof data14 == "number") {
													if (data14 > 9007199254740991 || isNaN(data14)) {
														validate39.errors = [{
															instancePath: instancePath + "/sequence",
															schemaPath: "#/properties/sequence/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 9007199254740991
															},
															message: "must be <= 9007199254740991"
														}];
														return false;
													} else if (data14 < 1 || isNaN(data14)) {
														validate39.errors = [{
															instancePath: instancePath + "/sequence",
															schemaPath: "#/properties/sequence/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 1
															},
															message: "must be >= 1"
														}];
														return false;
													}
												}
												var valid0 = true;
											} else var valid0 = true;
										}
									}
								}
							}
						}
					}
				}
			}
		} else {
			validate39.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate39.errors = vErrors;
		return true;
	}
	validate39.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.EmptyPayload = validate40;
	function validate40(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate40.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) for (const key0 in data) {
			validate40.errors = [{
				instancePath,
				schemaPath: "#/additionalProperties",
				keyword: "additionalProperties",
				params: { additionalProperty: key0 },
				message: "must NOT have additional properties"
			}];
			return false;
		}
		else {
			validate40.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate40.errors = vErrors;
		return true;
	}
	validate40.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.FailureEnvelope = validate41;
	function validate33(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate33.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate33.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate33.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.blocking !== void 0) {
					if (typeof data.blocking !== "boolean") {
						validate33.errors = [{
							instancePath: instancePath + "/blocking",
							schemaPath: "#/properties/blocking/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.category !== void 0) {
						if (typeof data.category !== "string") {
							validate33.errors = [{
								instancePath: instancePath + "/category",
								schemaPath: "#/properties/category/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.code !== void 0) {
							if (typeof data.code !== "string") {
								validate33.errors = [{
									instancePath: instancePath + "/code",
									schemaPath: "#/properties/code/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.message !== void 0) {
								if (typeof data.message !== "string") {
									validate33.errors = [{
										instancePath: instancePath + "/message",
										schemaPath: "#/properties/message/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.remediationActions !== void 0) {
									let data4 = data.remediationActions;
									if (Array.isArray(data4)) {
										const len0 = data4.length;
										for (let i0 = 0; i0 < len0; i0++) {
											let data5 = data4[i0];
											if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
												let missing1;
												if (data5.id === void 0 && (missing1 = "id") || data5.label === void 0 && (missing1 = "label")) {
													validate33.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate33.errors = [{
															instancePath: instancePath + "/remediationActions/" + i0,
															schemaPath: "#/$defs/Remediation/additionalProperties",
															keyword: "additionalProperties",
															params: { additionalProperty: key1 },
															message: "must NOT have additional properties"
														}];
														return false;
													}
													if (data5.id !== void 0) {
														if (typeof data5.id !== "string") {
															validate33.errors = [{
																instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																schemaPath: "#/$defs/Remediation/properties/id/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data5.label !== void 0) {
															if (typeof data5.label !== "string") {
																validate33.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/label",
																	schemaPath: "#/$defs/Remediation/properties/label/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
													}
												}
											} else {
												validate33.errors = [{
													instancePath: instancePath + "/remediationActions/" + i0,
													schemaPath: "#/$defs/Remediation/type",
													keyword: "type",
													params: { type: "object" },
													message: "must be object"
												}];
												return false;
											}
										}
									} else {
										validate33.errors = [{
											instancePath: instancePath + "/remediationActions",
											schemaPath: "#/properties/remediationActions/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.retryable !== void 0) {
										if (typeof data.retryable !== "boolean") {
											validate33.errors = [{
												instancePath: instancePath + "/retryable",
												schemaPath: "#/properties/retryable/type",
												keyword: "type",
												params: { type: "boolean" },
												message: "must be boolean"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			}
		} else {
			validate33.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate33.errors = vErrors;
		return true;
	}
	validate33.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate41(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate41.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate41.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "error" || key0 === "ok" || key0 === "requestId" || key0 === "schemaVersion")) {
						validate41.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.error !== void 0) {
							const _errs2 = errors;
							if (!validate33(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate33.errors : vErrors.concat(validate33.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate41.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate41.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/const",
										keyword: "const",
										params: { allowedValue: false },
										message: "must be equal to constant"
									}];
									return false;
								}
								var valid0 = _errs3 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.requestId !== void 0) {
									const _errs5 = errors;
									if (typeof data.requestId !== "string") {
										validate41.errors = [{
											instancePath: instancePath + "/requestId",
											schemaPath: "#/properties/requestId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate41.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate41.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/const",
												keyword: "const",
												params: { allowedValue: 1 },
												message: "must be equal to constant"
											}];
											return false;
										}
										if (errors === _errs7) {
											if (typeof data3 == "number") {
												if (data3 < 0 || isNaN(data3)) {
													validate41.errors = [{
														instancePath: instancePath + "/schemaVersion",
														schemaPath: "#/properties/schemaVersion/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													}];
													return false;
												}
											}
										}
										var valid0 = _errs7 === errors;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			} else {
				validate41.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate41.errors = vErrors;
		return errors === 0;
	}
	validate41.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.OpenWorkspace = validate43;
	var pattern4 = /* @__PURE__ */ new RegExp("^[A-Z]{3}$", "u");
	function validate43(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate43.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "name" || key0 === "path")) {
				validate43.errors = [{
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				}];
				return false;
			}
			if (data.baseCurrency !== void 0) {
				let data0 = data.baseCurrency;
				if (typeof data0 === "string") {
					if (!pattern4.test(data0)) {
						validate43.errors = [{
							instancePath: instancePath + "/baseCurrency",
							schemaPath: "#/properties/baseCurrency/pattern",
							keyword: "pattern",
							params: { pattern: "^[A-Z]{3}$" },
							message: "must match pattern \"^[A-Z]{3}$\""
						}];
						return false;
					}
				} else {
					validate43.errors = [{
						instancePath: instancePath + "/baseCurrency",
						schemaPath: "#/properties/baseCurrency/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					}];
					return false;
				}
				var valid0 = true;
			} else var valid0 = true;
			if (valid0) {
				if (data.name !== void 0) {
					let data1 = data.name;
					if (typeof data1 === "string") {
						if (func1(data1) > 120) {
							validate43.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/maxLength",
								keyword: "maxLength",
								params: { limit: 120 },
								message: "must NOT have more than 120 characters"
							}];
							return false;
						} else if (func1(data1) < 1) {
							validate43.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate43.errors = [{
							instancePath: instancePath + "/name",
							schemaPath: "#/properties/name/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.path !== void 0) {
						if (typeof data.path !== "string") {
							validate43.errors = [{
								instancePath: instancePath + "/path",
								schemaPath: "#/properties/path/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
				}
			}
		} else {
			validate43.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate43.errors = vErrors;
		return true;
	}
	validate43.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Remediation = validate44;
	function validate44(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate44.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.label === void 0 && (missing0 = "label")) {
				validate44.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "label")) {
					validate44.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.id !== void 0) {
					if (typeof data.id !== "string") {
						validate44.errors = [{
							instancePath: instancePath + "/id",
							schemaPath: "#/properties/id/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.label !== void 0) {
						if (typeof data.label !== "string") {
							validate44.errors = [{
								instancePath: instancePath + "/label",
								schemaPath: "#/properties/label/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
				}
			}
		} else {
			validate44.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate44.errors = vErrors;
		return true;
	}
	validate44.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ReplyData = validate45;
	function validate26(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate26.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
				validate26.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "projection")) {
					validate26.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.aggregateId !== void 0) {
					if (typeof data.aggregateId !== "string") {
						validate26.errors = [{
							instancePath: instancePath + "/aggregateId",
							schemaPath: "#/properties/aggregateId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateType !== void 0) {
						let data1 = data.aggregateType;
						if (typeof data1 !== "string") {
							validate26.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						if ("workspace" !== data1) {
							validate26.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/const",
								keyword: "const",
								params: { allowedValue: "workspace" },
								message: "must be equal to constant"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.lastSequence !== void 0) {
							let data2 = data.lastSequence;
							if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
								validate26.errors = [{
									instancePath: instancePath + "/lastSequence",
									schemaPath: "#/properties/lastSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								}];
								return false;
							}
							if (typeof data2 == "number") {
								if (data2 > 9007199254740991 || isNaN(data2)) {
									validate26.errors = [{
										instancePath: instancePath + "/lastSequence",
										schemaPath: "#/properties/lastSequence/maximum",
										keyword: "maximum",
										params: {
											comparison: "<=",
											limit: 9007199254740991
										},
										message: "must be <= 9007199254740991"
									}];
									return false;
								} else if (data2 < 0 || isNaN(data2)) {
									validate26.errors = [{
										instancePath: instancePath + "/lastSequence",
										schemaPath: "#/properties/lastSequence/minimum",
										keyword: "minimum",
										params: {
											comparison: ">=",
											limit: 0
										},
										message: "must be >= 0"
									}];
									return false;
								}
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.projection !== void 0) {
								let data3 = data.projection;
								if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
									let missing1;
									if (data3.workspaceId === void 0 && (missing1 = "workspaceId") || data3.name === void 0 && (missing1 = "name") || data3.baseCurrency === void 0 && (missing1 = "baseCurrency") || data3.path === void 0 && (missing1 = "path") || data3.createdAt === void 0 && (missing1 = "createdAt") || data3.lastOpenedAt === void 0 && (missing1 = "lastOpenedAt") || data3.storageSchemaVersion === void 0 && (missing1 = "storageSchemaVersion")) {
										validate26.errors = [{
											instancePath: instancePath + "/projection",
											schemaPath: "#/$defs/Workspace/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data3) if (!(key1 === "baseCurrency" || key1 === "createdAt" || key1 === "lastOpenedAt" || key1 === "name" || key1 === "path" || key1 === "storageSchemaVersion" || key1 === "workspaceId")) {
											validate26.errors = [{
												instancePath: instancePath + "/projection",
												schemaPath: "#/$defs/Workspace/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data3.baseCurrency !== void 0) {
											if (typeof data3.baseCurrency !== "string") {
												validate26.errors = [{
													instancePath: instancePath + "/projection/baseCurrency",
													schemaPath: "#/$defs/Workspace/properties/baseCurrency/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid2 = true;
										} else var valid2 = true;
										if (valid2) {
											if (data3.createdAt !== void 0) {
												if (typeof data3.createdAt !== "string") {
													validate26.errors = [{
														instancePath: instancePath + "/projection/createdAt",
														schemaPath: "#/$defs/Workspace/properties/createdAt/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid2 = true;
											} else var valid2 = true;
											if (valid2) {
												if (data3.lastOpenedAt !== void 0) {
													if (typeof data3.lastOpenedAt !== "string") {
														validate26.errors = [{
															instancePath: instancePath + "/projection/lastOpenedAt",
															schemaPath: "#/$defs/Workspace/properties/lastOpenedAt/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid2 = true;
												} else var valid2 = true;
												if (valid2) {
													if (data3.name !== void 0) {
														if (typeof data3.name !== "string") {
															validate26.errors = [{
																instancePath: instancePath + "/projection/name",
																schemaPath: "#/$defs/Workspace/properties/name/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid2 = true;
													} else var valid2 = true;
													if (valid2) {
														if (data3.path !== void 0) {
															if (typeof data3.path !== "string") {
																validate26.errors = [{
																	instancePath: instancePath + "/projection/path",
																	schemaPath: "#/$defs/Workspace/properties/path/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid2 = true;
														} else var valid2 = true;
														if (valid2) {
															if (data3.storageSchemaVersion !== void 0) {
																let data9 = data3.storageSchemaVersion;
																if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9))) {
																	validate26.errors = [{
																		instancePath: instancePath + "/projection/storageSchemaVersion",
																		schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/type",
																		keyword: "type",
																		params: { type: "integer" },
																		message: "must be integer"
																	}];
																	return false;
																}
																if (1 !== data9) {
																	validate26.errors = [{
																		instancePath: instancePath + "/projection/storageSchemaVersion",
																		schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/const",
																		keyword: "const",
																		params: { allowedValue: 1 },
																		message: "must be equal to constant"
																	}];
																	return false;
																}
																if (typeof data9 == "number") {
																	if (data9 < 0 || isNaN(data9)) {
																		validate26.errors = [{
																			instancePath: instancePath + "/projection/storageSchemaVersion",
																			schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																			keyword: "minimum",
																			params: {
																				comparison: ">=",
																				limit: 0
																			},
																			message: "must be >= 0"
																		}];
																		return false;
																	}
																}
																var valid2 = true;
															} else var valid2 = true;
															if (valid2) {
																if (data3.workspaceId !== void 0) {
																	if (typeof data3.workspaceId !== "string") {
																		validate26.errors = [{
																			instancePath: instancePath + "/projection/workspaceId",
																			schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid2 = true;
																} else var valid2 = true;
															}
														}
													}
												}
											}
										}
									}
								} else {
									validate26.errors = [{
										instancePath: instancePath + "/projection",
										schemaPath: "#/$defs/Workspace/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
						}
					}
				}
			}
		} else {
			validate26.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate26.errors = vErrors;
		return true;
	}
	validate26.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate28(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate28.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate28.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate28.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.components !== void 0) {
					let data0 = data.components;
					if (Array.isArray(data0)) {
						const len0 = data0.length;
						for (let i0 = 0; i0 < len0; i0++) {
							let data1 = data0[i0];
							if (data1 && typeof data1 == "object" && !Array.isArray(data1)) {
								let missing1;
								if (data1.id === void 0 && (missing1 = "id") || data1.status === void 0 && (missing1 = "status") || data1.message === void 0 && (missing1 = "message")) {
									validate28.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate28.errors = [{
											instancePath: instancePath + "/components/" + i0,
											schemaPath: "#/$defs/RuntimeComponent/additionalProperties",
											keyword: "additionalProperties",
											params: { additionalProperty: key1 },
											message: "must NOT have additional properties"
										}];
										return false;
									}
									if (data1.id !== void 0) {
										if (typeof data1.id !== "string") {
											validate28.errors = [{
												instancePath: instancePath + "/components/" + i0 + "/id",
												schemaPath: "#/$defs/RuntimeComponent/properties/id/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										var valid3 = true;
									} else var valid3 = true;
									if (valid3) {
										if (data1.message !== void 0) {
											if (typeof data1.message !== "string") {
												validate28.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/message",
													schemaPath: "#/$defs/RuntimeComponent/properties/message/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data1.status !== void 0) {
												if (typeof data1.status !== "string") {
													validate28.errors = [{
														instancePath: instancePath + "/components/" + i0 + "/status",
														schemaPath: "#/$defs/RuntimeComponent/properties/status/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid3 = true;
											} else var valid3 = true;
										}
									}
								}
							} else {
								validate28.errors = [{
									instancePath: instancePath + "/components/" + i0,
									schemaPath: "#/$defs/RuntimeComponent/type",
									keyword: "type",
									params: { type: "object" },
									message: "must be object"
								}];
								return false;
							}
						}
					} else {
						validate28.errors = [{
							instancePath: instancePath + "/components",
							schemaPath: "#/properties/components/type",
							keyword: "type",
							params: { type: "array" },
							message: "must be array"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.liveExecutionAvailable !== void 0) {
						if (typeof data.liveExecutionAvailable !== "boolean") {
							validate28.errors = [{
								instancePath: instancePath + "/liveExecutionAvailable",
								schemaPath: "#/properties/liveExecutionAvailable/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.modelAvailable !== void 0) {
							if (typeof data.modelAvailable !== "boolean") {
								validate28.errors = [{
									instancePath: instancePath + "/modelAvailable",
									schemaPath: "#/properties/modelAvailable/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate28.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate28.errors = vErrors;
		return true;
	}
	validate28.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate45(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate45.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.name === void 0 && (missing0 = "name") || data.baseCurrency === void 0 && (missing0 = "baseCurrency") || data.path === void 0 && (missing0 = "path") || data.createdAt === void 0 && (missing0 = "createdAt") || data.lastOpenedAt === void 0 && (missing0 = "lastOpenedAt") || data.storageSchemaVersion === void 0 && (missing0 = "storageSchemaVersion")) {
					const err0 = {
						instancePath,
						schemaPath: "#/$defs/Workspace/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					};
					if (vErrors === null) vErrors = [err0];
					else vErrors.push(err0);
					errors++;
				} else {
					const _errs4 = errors;
					for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "createdAt" || key0 === "lastOpenedAt" || key0 === "name" || key0 === "path" || key0 === "storageSchemaVersion" || key0 === "workspaceId")) {
						const err1 = {
							instancePath,
							schemaPath: "#/$defs/Workspace/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err1];
						else vErrors.push(err1);
						errors++;
						break;
					}
					if (_errs4 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs5 = errors;
							if (typeof data.baseCurrency !== "string") {
								const err2 = {
									instancePath: instancePath + "/baseCurrency",
									schemaPath: "#/$defs/Workspace/properties/baseCurrency/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								};
								if (vErrors === null) vErrors = [err2];
								else vErrors.push(err2);
								errors++;
							}
							var valid2 = _errs5 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs7 = errors;
								if (typeof data.createdAt !== "string") {
									const err3 = {
										instancePath: instancePath + "/createdAt",
										schemaPath: "#/$defs/Workspace/properties/createdAt/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err3];
									else vErrors.push(err3);
									errors++;
								}
								var valid2 = _errs7 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs9 = errors;
									if (typeof data.lastOpenedAt !== "string") {
										const err4 = {
											instancePath: instancePath + "/lastOpenedAt",
											schemaPath: "#/$defs/Workspace/properties/lastOpenedAt/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err4];
										else vErrors.push(err4);
										errors++;
									}
									var valid2 = _errs9 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs11 = errors;
										if (typeof data.name !== "string") {
											const err5 = {
												instancePath: instancePath + "/name",
												schemaPath: "#/$defs/Workspace/properties/name/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err5];
											else vErrors.push(err5);
											errors++;
										}
										var valid2 = _errs11 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs13 = errors;
											if (typeof data.path !== "string") {
												const err6 = {
													instancePath: instancePath + "/path",
													schemaPath: "#/$defs/Workspace/properties/path/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err6];
												else vErrors.push(err6);
												errors++;
											}
											var valid2 = _errs13 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs15 = errors;
												if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5))) {
													const err7 = {
														instancePath: instancePath + "/storageSchemaVersion",
														schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/type",
														keyword: "type",
														params: { type: "integer" },
														message: "must be integer"
													};
													if (vErrors === null) vErrors = [err7];
													else vErrors.push(err7);
													errors++;
												}
												if (1 !== data5) {
													const err8 = {
														instancePath: instancePath + "/storageSchemaVersion",
														schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/const",
														keyword: "const",
														params: { allowedValue: 1 },
														message: "must be equal to constant"
													};
													if (vErrors === null) vErrors = [err8];
													else vErrors.push(err8);
													errors++;
												}
												if (errors === _errs15) {
													if (typeof data5 == "number") {
														if (data5 < 0 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 0
																},
																message: "must be >= 0"
															};
															if (vErrors === null) vErrors = [err9];
															else vErrors.push(err9);
															errors++;
														}
													}
												}
												var valid2 = _errs15 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													const _errs17 = errors;
													if (typeof data.workspaceId !== "string") {
														const err10 = {
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														};
														if (vErrors === null) vErrors = [err10];
														else vErrors.push(err10);
														errors++;
													}
													var valid2 = _errs17 === errors;
												} else var valid2 = true;
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				const err11 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs19 = errors;
		if (!validate26(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate26.errors : vErrors.concat(validate26.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
		if (!validate28(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs20 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs21 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing1;
				if (data.aggregateType === void 0 && (missing1 = "aggregateType") || data.aggregateId === void 0 && (missing1 = "aggregateId") || data.afterSequence === void 0 && (missing1 = "afterSequence") || data.lastSequence === void 0 && (missing1 = "lastSequence") || data.replayedCount === void 0 && (missing1 = "replayedCount")) {
					const err12 = {
						instancePath,
						schemaPath: "#/$defs/SubscriptionAck/required",
						keyword: "required",
						params: { missingProperty: missing1 },
						message: "must have required property '" + missing1 + "'"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				} else {
					const _errs24 = errors;
					for (const key1 in data) if (!(key1 === "afterSequence" || key1 === "aggregateId" || key1 === "aggregateType" || key1 === "lastSequence" || key1 === "replayedCount")) {
						const err13 = {
							instancePath,
							schemaPath: "#/$defs/SubscriptionAck/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
						break;
					}
					if (_errs24 === errors) {
						if (data.afterSequence !== void 0) {
							let data7 = data.afterSequence;
							const _errs25 = errors;
							if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
								const err14 = {
									instancePath: instancePath + "/afterSequence",
									schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
							if (errors === _errs25) {
								if (typeof data7 == "number") {
									if (data7 > 9007199254740991 || isNaN(data7)) {
										const err15 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 9007199254740991
											},
											message: "must be <= 9007199254740991"
										};
										if (vErrors === null) vErrors = [err15];
										else vErrors.push(err15);
										errors++;
									} else if (data7 < 0 || isNaN(data7)) {
										const err16 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err16];
										else vErrors.push(err16);
										errors++;
									}
								}
							}
							var valid4 = _errs25 === errors;
						} else var valid4 = true;
						if (valid4) {
							if (data.aggregateId !== void 0) {
								const _errs27 = errors;
								if (typeof data.aggregateId !== "string") {
									const err17 = {
										instancePath: instancePath + "/aggregateId",
										schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err17];
									else vErrors.push(err17);
									errors++;
								}
								var valid4 = _errs27 === errors;
							} else var valid4 = true;
							if (valid4) {
								if (data.aggregateType !== void 0) {
									const _errs29 = errors;
									if (typeof data.aggregateType !== "string") {
										const err18 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err18];
										else vErrors.push(err18);
										errors++;
									}
									var valid4 = _errs29 === errors;
								} else var valid4 = true;
								if (valid4) {
									if (data.lastSequence !== void 0) {
										let data10 = data.lastSequence;
										const _errs31 = errors;
										if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
											const err19 = {
												instancePath: instancePath + "/lastSequence",
												schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											};
											if (vErrors === null) vErrors = [err19];
											else vErrors.push(err19);
											errors++;
										}
										if (errors === _errs31) {
											if (typeof data10 == "number") {
												if (data10 > 9007199254740991 || isNaN(data10)) {
													const err20 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 9007199254740991
														},
														message: "must be <= 9007199254740991"
													};
													if (vErrors === null) vErrors = [err20];
													else vErrors.push(err20);
													errors++;
												} else if (data10 < 0 || isNaN(data10)) {
													const err21 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													};
													if (vErrors === null) vErrors = [err21];
													else vErrors.push(err21);
													errors++;
												}
											}
										}
										var valid4 = _errs31 === errors;
									} else var valid4 = true;
									if (valid4) {
										if (data.replayedCount !== void 0) {
											let data11 = data.replayedCount;
											const _errs33 = errors;
											if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11))) {
												const err22 = {
													instancePath: instancePath + "/replayedCount",
													schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												};
												if (vErrors === null) vErrors = [err22];
												else vErrors.push(err22);
												errors++;
											}
											if (errors === _errs33) {
												if (typeof data11 == "number") {
													if (data11 > 9007199254740991 || isNaN(data11)) {
														const err23 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 9007199254740991
															},
															message: "must be <= 9007199254740991"
														};
														if (vErrors === null) vErrors = [err23];
														else vErrors.push(err23);
														errors++;
													} else if (data11 < 0 || isNaN(data11)) {
														const err24 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 0
															},
															message: "must be >= 0"
														};
														if (vErrors === null) vErrors = [err24];
														else vErrors.push(err24);
														errors++;
													}
												}
											}
											var valid4 = _errs33 === errors;
										} else var valid4 = true;
									}
								}
							}
						}
					}
				}
			} else {
				const err25 = {
					instancePath,
					schemaPath: "#/$defs/SubscriptionAck/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err25];
				else vErrors.push(err25);
				errors++;
			}
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err26 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err26];
			else vErrors.push(err26);
			errors++;
			validate45.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate45.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate45.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.ResultEnvelope = validate48;
	var schema38 = {
		"type": "object",
		"properties": {
			"data": { "$ref": "#/$defs/ReplyData" },
			"ok": {
				"type": "boolean",
				"const": true
			},
			"requestId": { "type": "string" },
			"schemaVersion": {
				"type": "integer",
				"format": "uint32",
				"const": 1,
				"minimum": 0
			},
			"stateVersion": { "type": ["string", "null"] }
		},
		"additionalProperties": false,
		"required": [
			"requestId",
			"schemaVersion",
			"ok",
			"data"
		]
	};
	function validate25(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate25.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.name === void 0 && (missing0 = "name") || data.baseCurrency === void 0 && (missing0 = "baseCurrency") || data.path === void 0 && (missing0 = "path") || data.createdAt === void 0 && (missing0 = "createdAt") || data.lastOpenedAt === void 0 && (missing0 = "lastOpenedAt") || data.storageSchemaVersion === void 0 && (missing0 = "storageSchemaVersion")) {
					const err0 = {
						instancePath,
						schemaPath: "#/$defs/Workspace/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					};
					if (vErrors === null) vErrors = [err0];
					else vErrors.push(err0);
					errors++;
				} else {
					const _errs4 = errors;
					for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "createdAt" || key0 === "lastOpenedAt" || key0 === "name" || key0 === "path" || key0 === "storageSchemaVersion" || key0 === "workspaceId")) {
						const err1 = {
							instancePath,
							schemaPath: "#/$defs/Workspace/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err1];
						else vErrors.push(err1);
						errors++;
						break;
					}
					if (_errs4 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs5 = errors;
							if (typeof data.baseCurrency !== "string") {
								const err2 = {
									instancePath: instancePath + "/baseCurrency",
									schemaPath: "#/$defs/Workspace/properties/baseCurrency/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								};
								if (vErrors === null) vErrors = [err2];
								else vErrors.push(err2);
								errors++;
							}
							var valid2 = _errs5 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs7 = errors;
								if (typeof data.createdAt !== "string") {
									const err3 = {
										instancePath: instancePath + "/createdAt",
										schemaPath: "#/$defs/Workspace/properties/createdAt/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err3];
									else vErrors.push(err3);
									errors++;
								}
								var valid2 = _errs7 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs9 = errors;
									if (typeof data.lastOpenedAt !== "string") {
										const err4 = {
											instancePath: instancePath + "/lastOpenedAt",
											schemaPath: "#/$defs/Workspace/properties/lastOpenedAt/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err4];
										else vErrors.push(err4);
										errors++;
									}
									var valid2 = _errs9 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs11 = errors;
										if (typeof data.name !== "string") {
											const err5 = {
												instancePath: instancePath + "/name",
												schemaPath: "#/$defs/Workspace/properties/name/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err5];
											else vErrors.push(err5);
											errors++;
										}
										var valid2 = _errs11 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs13 = errors;
											if (typeof data.path !== "string") {
												const err6 = {
													instancePath: instancePath + "/path",
													schemaPath: "#/$defs/Workspace/properties/path/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err6];
												else vErrors.push(err6);
												errors++;
											}
											var valid2 = _errs13 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs15 = errors;
												if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5))) {
													const err7 = {
														instancePath: instancePath + "/storageSchemaVersion",
														schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/type",
														keyword: "type",
														params: { type: "integer" },
														message: "must be integer"
													};
													if (vErrors === null) vErrors = [err7];
													else vErrors.push(err7);
													errors++;
												}
												if (1 !== data5) {
													const err8 = {
														instancePath: instancePath + "/storageSchemaVersion",
														schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/const",
														keyword: "const",
														params: { allowedValue: 1 },
														message: "must be equal to constant"
													};
													if (vErrors === null) vErrors = [err8];
													else vErrors.push(err8);
													errors++;
												}
												if (errors === _errs15) {
													if (typeof data5 == "number") {
														if (data5 < 0 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 0
																},
																message: "must be >= 0"
															};
															if (vErrors === null) vErrors = [err9];
															else vErrors.push(err9);
															errors++;
														}
													}
												}
												var valid2 = _errs15 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													const _errs17 = errors;
													if (typeof data.workspaceId !== "string") {
														const err10 = {
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														};
														if (vErrors === null) vErrors = [err10];
														else vErrors.push(err10);
														errors++;
													}
													var valid2 = _errs17 === errors;
												} else var valid2 = true;
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				const err11 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs19 = errors;
		if (!validate26(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate26.errors : vErrors.concat(validate26.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
		if (!validate28(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs20 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs21 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing1;
				if (data.aggregateType === void 0 && (missing1 = "aggregateType") || data.aggregateId === void 0 && (missing1 = "aggregateId") || data.afterSequence === void 0 && (missing1 = "afterSequence") || data.lastSequence === void 0 && (missing1 = "lastSequence") || data.replayedCount === void 0 && (missing1 = "replayedCount")) {
					const err12 = {
						instancePath,
						schemaPath: "#/$defs/SubscriptionAck/required",
						keyword: "required",
						params: { missingProperty: missing1 },
						message: "must have required property '" + missing1 + "'"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				} else {
					const _errs24 = errors;
					for (const key1 in data) if (!(key1 === "afterSequence" || key1 === "aggregateId" || key1 === "aggregateType" || key1 === "lastSequence" || key1 === "replayedCount")) {
						const err13 = {
							instancePath,
							schemaPath: "#/$defs/SubscriptionAck/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
						break;
					}
					if (_errs24 === errors) {
						if (data.afterSequence !== void 0) {
							let data7 = data.afterSequence;
							const _errs25 = errors;
							if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
								const err14 = {
									instancePath: instancePath + "/afterSequence",
									schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
							if (errors === _errs25) {
								if (typeof data7 == "number") {
									if (data7 > 9007199254740991 || isNaN(data7)) {
										const err15 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 9007199254740991
											},
											message: "must be <= 9007199254740991"
										};
										if (vErrors === null) vErrors = [err15];
										else vErrors.push(err15);
										errors++;
									} else if (data7 < 0 || isNaN(data7)) {
										const err16 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err16];
										else vErrors.push(err16);
										errors++;
									}
								}
							}
							var valid4 = _errs25 === errors;
						} else var valid4 = true;
						if (valid4) {
							if (data.aggregateId !== void 0) {
								const _errs27 = errors;
								if (typeof data.aggregateId !== "string") {
									const err17 = {
										instancePath: instancePath + "/aggregateId",
										schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err17];
									else vErrors.push(err17);
									errors++;
								}
								var valid4 = _errs27 === errors;
							} else var valid4 = true;
							if (valid4) {
								if (data.aggregateType !== void 0) {
									const _errs29 = errors;
									if (typeof data.aggregateType !== "string") {
										const err18 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err18];
										else vErrors.push(err18);
										errors++;
									}
									var valid4 = _errs29 === errors;
								} else var valid4 = true;
								if (valid4) {
									if (data.lastSequence !== void 0) {
										let data10 = data.lastSequence;
										const _errs31 = errors;
										if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
											const err19 = {
												instancePath: instancePath + "/lastSequence",
												schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											};
											if (vErrors === null) vErrors = [err19];
											else vErrors.push(err19);
											errors++;
										}
										if (errors === _errs31) {
											if (typeof data10 == "number") {
												if (data10 > 9007199254740991 || isNaN(data10)) {
													const err20 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 9007199254740991
														},
														message: "must be <= 9007199254740991"
													};
													if (vErrors === null) vErrors = [err20];
													else vErrors.push(err20);
													errors++;
												} else if (data10 < 0 || isNaN(data10)) {
													const err21 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													};
													if (vErrors === null) vErrors = [err21];
													else vErrors.push(err21);
													errors++;
												}
											}
										}
										var valid4 = _errs31 === errors;
									} else var valid4 = true;
									if (valid4) {
										if (data.replayedCount !== void 0) {
											let data11 = data.replayedCount;
											const _errs33 = errors;
											if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11))) {
												const err22 = {
													instancePath: instancePath + "/replayedCount",
													schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												};
												if (vErrors === null) vErrors = [err22];
												else vErrors.push(err22);
												errors++;
											}
											if (errors === _errs33) {
												if (typeof data11 == "number") {
													if (data11 > 9007199254740991 || isNaN(data11)) {
														const err23 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 9007199254740991
															},
															message: "must be <= 9007199254740991"
														};
														if (vErrors === null) vErrors = [err23];
														else vErrors.push(err23);
														errors++;
													} else if (data11 < 0 || isNaN(data11)) {
														const err24 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 0
															},
															message: "must be >= 0"
														};
														if (vErrors === null) vErrors = [err24];
														else vErrors.push(err24);
														errors++;
													}
												}
											}
											var valid4 = _errs33 === errors;
										} else var valid4 = true;
									}
								}
							}
						}
					}
				}
			} else {
				const err25 = {
					instancePath,
					schemaPath: "#/$defs/SubscriptionAck/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err25];
				else vErrors.push(err25);
				errors++;
			}
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err26 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err26];
			else vErrors.push(err26);
			errors++;
			validate25.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate25.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate25.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate24(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate24.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate24.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "data" || key0 === "ok" || key0 === "requestId" || key0 === "schemaVersion" || key0 === "stateVersion")) {
						validate24.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.data !== void 0) {
							const _errs2 = errors;
							if (!validate25(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate24.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate24.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/const",
										keyword: "const",
										params: { allowedValue: true },
										message: "must be equal to constant"
									}];
									return false;
								}
								var valid0 = _errs3 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.requestId !== void 0) {
									const _errs5 = errors;
									if (typeof data.requestId !== "string") {
										validate24.errors = [{
											instancePath: instancePath + "/requestId",
											schemaPath: "#/properties/requestId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate24.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate24.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/const",
												keyword: "const",
												params: { allowedValue: 1 },
												message: "must be equal to constant"
											}];
											return false;
										}
										if (errors === _errs7) {
											if (typeof data3 == "number") {
												if (data3 < 0 || isNaN(data3)) {
													validate24.errors = [{
														instancePath: instancePath + "/schemaVersion",
														schemaPath: "#/properties/schemaVersion/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													}];
													return false;
												}
											}
										}
										var valid0 = _errs7 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.stateVersion !== void 0) {
											let data4 = data.stateVersion;
											const _errs9 = errors;
											if (typeof data4 !== "string" && data4 !== null) {
												validate24.errors = [{
													instancePath: instancePath + "/stateVersion",
													schemaPath: "#/properties/stateVersion/type",
													keyword: "type",
													params: { type: schema38.properties.stateVersion.type },
													message: "must be string,null"
												}];
												return false;
											}
											var valid0 = _errs9 === errors;
										} else var valid0 = true;
									}
								}
							}
						}
					}
				}
			} else {
				validate24.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate24.errors = vErrors;
		return errors === 0;
	}
	validate24.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate32(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate32.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate32.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "error" || key0 === "ok" || key0 === "requestId" || key0 === "schemaVersion")) {
						validate32.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.error !== void 0) {
							const _errs2 = errors;
							if (!validate33(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate33.errors : vErrors.concat(validate33.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate32.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate32.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/const",
										keyword: "const",
										params: { allowedValue: false },
										message: "must be equal to constant"
									}];
									return false;
								}
								var valid0 = _errs3 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.requestId !== void 0) {
									const _errs5 = errors;
									if (typeof data.requestId !== "string") {
										validate32.errors = [{
											instancePath: instancePath + "/requestId",
											schemaPath: "#/properties/requestId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate32.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate32.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/const",
												keyword: "const",
												params: { allowedValue: 1 },
												message: "must be equal to constant"
											}];
											return false;
										}
										if (errors === _errs7) {
											if (typeof data3 == "number") {
												if (data3 < 0 || isNaN(data3)) {
													validate32.errors = [{
														instancePath: instancePath + "/schemaVersion",
														schemaPath: "#/properties/schemaVersion/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													}];
													return false;
												}
											}
										}
										var valid0 = _errs7 === errors;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			} else {
				validate32.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate32.errors = vErrors;
		return errors === 0;
	}
	validate32.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate48(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate48.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate24(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
		if (!validate32(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
			validate48.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate48.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate48.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.RuntimeComponent = validate51;
	function validate51(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate51.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.status === void 0 && (missing0 = "status") || data.message === void 0 && (missing0 = "message")) {
				validate51.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "message" || key0 === "status")) {
					validate51.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.id !== void 0) {
					if (typeof data.id !== "string") {
						validate51.errors = [{
							instancePath: instancePath + "/id",
							schemaPath: "#/properties/id/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.message !== void 0) {
						if (typeof data.message !== "string") {
							validate51.errors = [{
								instancePath: instancePath + "/message",
								schemaPath: "#/properties/message/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.status !== void 0) {
							if (typeof data.status !== "string") {
								validate51.errors = [{
									instancePath: instancePath + "/status",
									schemaPath: "#/properties/status/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate51.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate51.errors = vErrors;
		return true;
	}
	validate51.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RuntimeStatus = validate52;
	function validate52(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate52.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate52.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate52.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.components !== void 0) {
					let data0 = data.components;
					if (Array.isArray(data0)) {
						const len0 = data0.length;
						for (let i0 = 0; i0 < len0; i0++) {
							let data1 = data0[i0];
							if (data1 && typeof data1 == "object" && !Array.isArray(data1)) {
								let missing1;
								if (data1.id === void 0 && (missing1 = "id") || data1.status === void 0 && (missing1 = "status") || data1.message === void 0 && (missing1 = "message")) {
									validate52.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate52.errors = [{
											instancePath: instancePath + "/components/" + i0,
											schemaPath: "#/$defs/RuntimeComponent/additionalProperties",
											keyword: "additionalProperties",
											params: { additionalProperty: key1 },
											message: "must NOT have additional properties"
										}];
										return false;
									}
									if (data1.id !== void 0) {
										if (typeof data1.id !== "string") {
											validate52.errors = [{
												instancePath: instancePath + "/components/" + i0 + "/id",
												schemaPath: "#/$defs/RuntimeComponent/properties/id/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										var valid3 = true;
									} else var valid3 = true;
									if (valid3) {
										if (data1.message !== void 0) {
											if (typeof data1.message !== "string") {
												validate52.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/message",
													schemaPath: "#/$defs/RuntimeComponent/properties/message/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data1.status !== void 0) {
												if (typeof data1.status !== "string") {
													validate52.errors = [{
														instancePath: instancePath + "/components/" + i0 + "/status",
														schemaPath: "#/$defs/RuntimeComponent/properties/status/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid3 = true;
											} else var valid3 = true;
										}
									}
								}
							} else {
								validate52.errors = [{
									instancePath: instancePath + "/components/" + i0,
									schemaPath: "#/$defs/RuntimeComponent/type",
									keyword: "type",
									params: { type: "object" },
									message: "must be object"
								}];
								return false;
							}
						}
					} else {
						validate52.errors = [{
							instancePath: instancePath + "/components",
							schemaPath: "#/properties/components/type",
							keyword: "type",
							params: { type: "array" },
							message: "must be array"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.liveExecutionAvailable !== void 0) {
						if (typeof data.liveExecutionAvailable !== "boolean") {
							validate52.errors = [{
								instancePath: instancePath + "/liveExecutionAvailable",
								schemaPath: "#/properties/liveExecutionAvailable/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.modelAvailable !== void 0) {
							if (typeof data.modelAvailable !== "boolean") {
								validate52.errors = [{
									instancePath: instancePath + "/modelAvailable",
									schemaPath: "#/properties/modelAvailable/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate52.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate52.errors = vErrors;
		return true;
	}
	validate52.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Snapshot = validate53;
	function validate53(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate53.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
				validate53.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "projection")) {
					validate53.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.aggregateId !== void 0) {
					if (typeof data.aggregateId !== "string") {
						validate53.errors = [{
							instancePath: instancePath + "/aggregateId",
							schemaPath: "#/properties/aggregateId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateType !== void 0) {
						let data1 = data.aggregateType;
						if (typeof data1 !== "string") {
							validate53.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						if ("workspace" !== data1) {
							validate53.errors = [{
								instancePath: instancePath + "/aggregateType",
								schemaPath: "#/properties/aggregateType/const",
								keyword: "const",
								params: { allowedValue: "workspace" },
								message: "must be equal to constant"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.lastSequence !== void 0) {
							let data2 = data.lastSequence;
							if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
								validate53.errors = [{
									instancePath: instancePath + "/lastSequence",
									schemaPath: "#/properties/lastSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								}];
								return false;
							}
							if (typeof data2 == "number") {
								if (data2 > 9007199254740991 || isNaN(data2)) {
									validate53.errors = [{
										instancePath: instancePath + "/lastSequence",
										schemaPath: "#/properties/lastSequence/maximum",
										keyword: "maximum",
										params: {
											comparison: "<=",
											limit: 9007199254740991
										},
										message: "must be <= 9007199254740991"
									}];
									return false;
								} else if (data2 < 0 || isNaN(data2)) {
									validate53.errors = [{
										instancePath: instancePath + "/lastSequence",
										schemaPath: "#/properties/lastSequence/minimum",
										keyword: "minimum",
										params: {
											comparison: ">=",
											limit: 0
										},
										message: "must be >= 0"
									}];
									return false;
								}
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.projection !== void 0) {
								let data3 = data.projection;
								if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
									let missing1;
									if (data3.workspaceId === void 0 && (missing1 = "workspaceId") || data3.name === void 0 && (missing1 = "name") || data3.baseCurrency === void 0 && (missing1 = "baseCurrency") || data3.path === void 0 && (missing1 = "path") || data3.createdAt === void 0 && (missing1 = "createdAt") || data3.lastOpenedAt === void 0 && (missing1 = "lastOpenedAt") || data3.storageSchemaVersion === void 0 && (missing1 = "storageSchemaVersion")) {
										validate53.errors = [{
											instancePath: instancePath + "/projection",
											schemaPath: "#/$defs/Workspace/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data3) if (!(key1 === "baseCurrency" || key1 === "createdAt" || key1 === "lastOpenedAt" || key1 === "name" || key1 === "path" || key1 === "storageSchemaVersion" || key1 === "workspaceId")) {
											validate53.errors = [{
												instancePath: instancePath + "/projection",
												schemaPath: "#/$defs/Workspace/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data3.baseCurrency !== void 0) {
											if (typeof data3.baseCurrency !== "string") {
												validate53.errors = [{
													instancePath: instancePath + "/projection/baseCurrency",
													schemaPath: "#/$defs/Workspace/properties/baseCurrency/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid2 = true;
										} else var valid2 = true;
										if (valid2) {
											if (data3.createdAt !== void 0) {
												if (typeof data3.createdAt !== "string") {
													validate53.errors = [{
														instancePath: instancePath + "/projection/createdAt",
														schemaPath: "#/$defs/Workspace/properties/createdAt/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid2 = true;
											} else var valid2 = true;
											if (valid2) {
												if (data3.lastOpenedAt !== void 0) {
													if (typeof data3.lastOpenedAt !== "string") {
														validate53.errors = [{
															instancePath: instancePath + "/projection/lastOpenedAt",
															schemaPath: "#/$defs/Workspace/properties/lastOpenedAt/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid2 = true;
												} else var valid2 = true;
												if (valid2) {
													if (data3.name !== void 0) {
														if (typeof data3.name !== "string") {
															validate53.errors = [{
																instancePath: instancePath + "/projection/name",
																schemaPath: "#/$defs/Workspace/properties/name/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid2 = true;
													} else var valid2 = true;
													if (valid2) {
														if (data3.path !== void 0) {
															if (typeof data3.path !== "string") {
																validate53.errors = [{
																	instancePath: instancePath + "/projection/path",
																	schemaPath: "#/$defs/Workspace/properties/path/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid2 = true;
														} else var valid2 = true;
														if (valid2) {
															if (data3.storageSchemaVersion !== void 0) {
																let data9 = data3.storageSchemaVersion;
																if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9))) {
																	validate53.errors = [{
																		instancePath: instancePath + "/projection/storageSchemaVersion",
																		schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/type",
																		keyword: "type",
																		params: { type: "integer" },
																		message: "must be integer"
																	}];
																	return false;
																}
																if (1 !== data9) {
																	validate53.errors = [{
																		instancePath: instancePath + "/projection/storageSchemaVersion",
																		schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/const",
																		keyword: "const",
																		params: { allowedValue: 1 },
																		message: "must be equal to constant"
																	}];
																	return false;
																}
																if (typeof data9 == "number") {
																	if (data9 < 0 || isNaN(data9)) {
																		validate53.errors = [{
																			instancePath: instancePath + "/projection/storageSchemaVersion",
																			schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																			keyword: "minimum",
																			params: {
																				comparison: ">=",
																				limit: 0
																			},
																			message: "must be >= 0"
																		}];
																		return false;
																	}
																}
																var valid2 = true;
															} else var valid2 = true;
															if (valid2) {
																if (data3.workspaceId !== void 0) {
																	if (typeof data3.workspaceId !== "string") {
																		validate53.errors = [{
																			instancePath: instancePath + "/projection/workspaceId",
																			schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid2 = true;
																} else var valid2 = true;
															}
														}
													}
												}
											}
										}
									}
								} else {
									validate53.errors = [{
										instancePath: instancePath + "/projection",
										schemaPath: "#/$defs/Workspace/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
						}
					}
				}
			}
		} else {
			validate53.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate53.errors = vErrors;
		return true;
	}
	validate53.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Subscribe = validate54;
	function validate54(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate54.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence")) {
				validate54.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType")) {
					validate54.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.afterSequence !== void 0) {
					let data0 = data.afterSequence;
					if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0))) {
						validate54.errors = [{
							instancePath: instancePath + "/afterSequence",
							schemaPath: "#/properties/afterSequence/type",
							keyword: "type",
							params: { type: "integer" },
							message: "must be integer"
						}];
						return false;
					}
					if (typeof data0 == "number") {
						if (data0 > 9007199254740991 || isNaN(data0)) {
							validate54.errors = [{
								instancePath: instancePath + "/afterSequence",
								schemaPath: "#/properties/afterSequence/maximum",
								keyword: "maximum",
								params: {
									comparison: "<=",
									limit: 9007199254740991
								},
								message: "must be <= 9007199254740991"
							}];
							return false;
						} else if (data0 < 0 || isNaN(data0)) {
							validate54.errors = [{
								instancePath: instancePath + "/afterSequence",
								schemaPath: "#/properties/afterSequence/minimum",
								keyword: "minimum",
								params: {
									comparison: ">=",
									limit: 0
								},
								message: "must be >= 0"
							}];
							return false;
						}
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateId !== void 0) {
						if (typeof data.aggregateId !== "string") {
							validate54.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.aggregateType !== void 0) {
							if (typeof data.aggregateType !== "string") {
								validate54.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate54.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate54.errors = vErrors;
		return true;
	}
	validate54.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SubscriptionAck = validate55;
	function validate55(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate55.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence") || data.lastSequence === void 0 && (missing0 = "lastSequence") || data.replayedCount === void 0 && (missing0 = "replayedCount")) {
				validate55.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "replayedCount")) {
					validate55.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.afterSequence !== void 0) {
					let data0 = data.afterSequence;
					if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0))) {
						validate55.errors = [{
							instancePath: instancePath + "/afterSequence",
							schemaPath: "#/properties/afterSequence/type",
							keyword: "type",
							params: { type: "integer" },
							message: "must be integer"
						}];
						return false;
					}
					if (typeof data0 == "number") {
						if (data0 > 9007199254740991 || isNaN(data0)) {
							validate55.errors = [{
								instancePath: instancePath + "/afterSequence",
								schemaPath: "#/properties/afterSequence/maximum",
								keyword: "maximum",
								params: {
									comparison: "<=",
									limit: 9007199254740991
								},
								message: "must be <= 9007199254740991"
							}];
							return false;
						} else if (data0 < 0 || isNaN(data0)) {
							validate55.errors = [{
								instancePath: instancePath + "/afterSequence",
								schemaPath: "#/properties/afterSequence/minimum",
								keyword: "minimum",
								params: {
									comparison: ">=",
									limit: 0
								},
								message: "must be >= 0"
							}];
							return false;
						}
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.aggregateId !== void 0) {
						if (typeof data.aggregateId !== "string") {
							validate55.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.aggregateType !== void 0) {
							if (typeof data.aggregateType !== "string") {
								validate55.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.lastSequence !== void 0) {
								let data3 = data.lastSequence;
								if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
									validate55.errors = [{
										instancePath: instancePath + "/lastSequence",
										schemaPath: "#/properties/lastSequence/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									}];
									return false;
								}
								if (typeof data3 == "number") {
									if (data3 > 9007199254740991 || isNaN(data3)) {
										validate55.errors = [{
											instancePath: instancePath + "/lastSequence",
											schemaPath: "#/properties/lastSequence/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 9007199254740991
											},
											message: "must be <= 9007199254740991"
										}];
										return false;
									} else if (data3 < 0 || isNaN(data3)) {
										validate55.errors = [{
											instancePath: instancePath + "/lastSequence",
											schemaPath: "#/properties/lastSequence/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										}];
										return false;
									}
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.replayedCount !== void 0) {
									let data4 = data.replayedCount;
									if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4))) {
										validate55.errors = [{
											instancePath: instancePath + "/replayedCount",
											schemaPath: "#/properties/replayedCount/type",
											keyword: "type",
											params: { type: "integer" },
											message: "must be integer"
										}];
										return false;
									}
									if (typeof data4 == "number") {
										if (data4 > 9007199254740991 || isNaN(data4)) {
											validate55.errors = [{
												instancePath: instancePath + "/replayedCount",
												schemaPath: "#/properties/replayedCount/maximum",
												keyword: "maximum",
												params: {
													comparison: "<=",
													limit: 9007199254740991
												},
												message: "must be <= 9007199254740991"
											}];
											return false;
										} else if (data4 < 0 || isNaN(data4)) {
											validate55.errors = [{
												instancePath: instancePath + "/replayedCount",
												schemaPath: "#/properties/replayedCount/minimum",
												keyword: "minimum",
												params: {
													comparison: ">=",
													limit: 0
												},
												message: "must be >= 0"
											}];
											return false;
										}
									}
									var valid0 = true;
								} else var valid0 = true;
							}
						}
					}
				}
			}
		} else {
			validate55.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate55.errors = vErrors;
		return true;
	}
	validate55.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SuccessEnvelope = validate56;
	function validate56(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate56.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate56.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "data" || key0 === "ok" || key0 === "requestId" || key0 === "schemaVersion" || key0 === "stateVersion")) {
						validate56.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.data !== void 0) {
							const _errs2 = errors;
							if (!validate25(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate56.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate56.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/const",
										keyword: "const",
										params: { allowedValue: true },
										message: "must be equal to constant"
									}];
									return false;
								}
								var valid0 = _errs3 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.requestId !== void 0) {
									const _errs5 = errors;
									if (typeof data.requestId !== "string") {
										validate56.errors = [{
											instancePath: instancePath + "/requestId",
											schemaPath: "#/properties/requestId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate56.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate56.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/const",
												keyword: "const",
												params: { allowedValue: 1 },
												message: "must be equal to constant"
											}];
											return false;
										}
										if (errors === _errs7) {
											if (typeof data3 == "number") {
												if (data3 < 0 || isNaN(data3)) {
													validate56.errors = [{
														instancePath: instancePath + "/schemaVersion",
														schemaPath: "#/properties/schemaVersion/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 0
														},
														message: "must be >= 0"
													}];
													return false;
												}
											}
										}
										var valid0 = _errs7 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.stateVersion !== void 0) {
											let data4 = data.stateVersion;
											const _errs9 = errors;
											if (typeof data4 !== "string" && data4 !== null) {
												validate56.errors = [{
													instancePath: instancePath + "/stateVersion",
													schemaPath: "#/properties/stateVersion/type",
													keyword: "type",
													params: { type: schema38.properties.stateVersion.type },
													message: "must be string,null"
												}];
												return false;
											}
											var valid0 = _errs9 === errors;
										} else var valid0 = true;
									}
								}
							}
						}
					}
				}
			} else {
				validate56.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate56.errors = vErrors;
		return errors === 0;
	}
	validate56.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.TradeXError = validate58;
	function validate58(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate58.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate58.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate58.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.blocking !== void 0) {
					if (typeof data.blocking !== "boolean") {
						validate58.errors = [{
							instancePath: instancePath + "/blocking",
							schemaPath: "#/properties/blocking/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.category !== void 0) {
						if (typeof data.category !== "string") {
							validate58.errors = [{
								instancePath: instancePath + "/category",
								schemaPath: "#/properties/category/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.code !== void 0) {
							if (typeof data.code !== "string") {
								validate58.errors = [{
									instancePath: instancePath + "/code",
									schemaPath: "#/properties/code/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.message !== void 0) {
								if (typeof data.message !== "string") {
									validate58.errors = [{
										instancePath: instancePath + "/message",
										schemaPath: "#/properties/message/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.remediationActions !== void 0) {
									let data4 = data.remediationActions;
									if (Array.isArray(data4)) {
										const len0 = data4.length;
										for (let i0 = 0; i0 < len0; i0++) {
											let data5 = data4[i0];
											if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
												let missing1;
												if (data5.id === void 0 && (missing1 = "id") || data5.label === void 0 && (missing1 = "label")) {
													validate58.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate58.errors = [{
															instancePath: instancePath + "/remediationActions/" + i0,
															schemaPath: "#/$defs/Remediation/additionalProperties",
															keyword: "additionalProperties",
															params: { additionalProperty: key1 },
															message: "must NOT have additional properties"
														}];
														return false;
													}
													if (data5.id !== void 0) {
														if (typeof data5.id !== "string") {
															validate58.errors = [{
																instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																schemaPath: "#/$defs/Remediation/properties/id/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data5.label !== void 0) {
															if (typeof data5.label !== "string") {
																validate58.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/label",
																	schemaPath: "#/$defs/Remediation/properties/label/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
													}
												}
											} else {
												validate58.errors = [{
													instancePath: instancePath + "/remediationActions/" + i0,
													schemaPath: "#/$defs/Remediation/type",
													keyword: "type",
													params: { type: "object" },
													message: "must be object"
												}];
												return false;
											}
										}
									} else {
										validate58.errors = [{
											instancePath: instancePath + "/remediationActions",
											schemaPath: "#/properties/remediationActions/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.retryable !== void 0) {
										if (typeof data.retryable !== "boolean") {
											validate58.errors = [{
												instancePath: instancePath + "/retryable",
												schemaPath: "#/properties/retryable/type",
												keyword: "type",
												params: { type: "boolean" },
												message: "must be boolean"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			}
		} else {
			validate58.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate58.errors = vErrors;
		return true;
	}
	validate58.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Workspace = validate59;
	function validate59(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate59.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.name === void 0 && (missing0 = "name") || data.baseCurrency === void 0 && (missing0 = "baseCurrency") || data.path === void 0 && (missing0 = "path") || data.createdAt === void 0 && (missing0 = "createdAt") || data.lastOpenedAt === void 0 && (missing0 = "lastOpenedAt") || data.storageSchemaVersion === void 0 && (missing0 = "storageSchemaVersion")) {
				validate59.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "createdAt" || key0 === "lastOpenedAt" || key0 === "name" || key0 === "path" || key0 === "storageSchemaVersion" || key0 === "workspaceId")) {
					validate59.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.baseCurrency !== void 0) {
					if (typeof data.baseCurrency !== "string") {
						validate59.errors = [{
							instancePath: instancePath + "/baseCurrency",
							schemaPath: "#/properties/baseCurrency/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.createdAt !== void 0) {
						if (typeof data.createdAt !== "string") {
							validate59.errors = [{
								instancePath: instancePath + "/createdAt",
								schemaPath: "#/properties/createdAt/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.lastOpenedAt !== void 0) {
							if (typeof data.lastOpenedAt !== "string") {
								validate59.errors = [{
									instancePath: instancePath + "/lastOpenedAt",
									schemaPath: "#/properties/lastOpenedAt/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.name !== void 0) {
								if (typeof data.name !== "string") {
									validate59.errors = [{
										instancePath: instancePath + "/name",
										schemaPath: "#/properties/name/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.path !== void 0) {
									if (typeof data.path !== "string") {
										validate59.errors = [{
											instancePath: instancePath + "/path",
											schemaPath: "#/properties/path/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.storageSchemaVersion !== void 0) {
										let data5 = data.storageSchemaVersion;
										if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5))) {
											validate59.errors = [{
												instancePath: instancePath + "/storageSchemaVersion",
												schemaPath: "#/properties/storageSchemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data5) {
											validate59.errors = [{
												instancePath: instancePath + "/storageSchemaVersion",
												schemaPath: "#/properties/storageSchemaVersion/const",
												keyword: "const",
												params: { allowedValue: 1 },
												message: "must be equal to constant"
											}];
											return false;
										}
										if (typeof data5 == "number") {
											if (data5 < 0 || isNaN(data5)) {
												validate59.errors = [{
													instancePath: instancePath + "/storageSchemaVersion",
													schemaPath: "#/properties/storageSchemaVersion/minimum",
													keyword: "minimum",
													params: {
														comparison: ">=",
														limit: 0
													},
													message: "must be >= 0"
												}];
												return false;
											}
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.workspaceId !== void 0) {
											if (typeof data.workspaceId !== "string") {
												validate59.errors = [{
													instancePath: instancePath + "/workspaceId",
													schemaPath: "#/properties/workspaceId/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
									}
								}
							}
						}
					}
				}
			}
		} else {
			validate59.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate59.errors = vErrors;
		return true;
	}
	validate59.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
}));
//#endregion
export default require_ipc_validators_input();
