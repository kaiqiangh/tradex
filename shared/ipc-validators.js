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
	exports.AccountConnection = validate82;
	var schema62 = {
		"type": "object",
		"properties": {
			"connectionId": {
				"type": "string",
				"minLength": 1
			},
			"connectionState": { "$ref": "#/$defs/ConnectionState" },
			"createdAt": { "type": "string" },
			"data": { "anyOf": [{ "$ref": "#/$defs/AccountData" }, { "type": "null" }] },
			"environment": { "type": "string" },
			"health": { "$ref": "#/$defs/AccountHealth" },
			"label": { "type": "string" },
			"lastSuccessfulSync": { "type": ["string", "null"] },
			"permissions": { "$ref": "#/$defs/PermissionReview" },
			"providerId": { "type": "string" },
			"stateVersion": {
				"type": "string",
				"minLength": 1
			},
			"updatedAt": { "type": "string" },
			"workspaceId": {
				"type": "string",
				"minLength": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"connectionId",
			"workspaceId",
			"providerId",
			"environment",
			"label",
			"createdAt",
			"updatedAt",
			"stateVersion",
			"connectionState",
			"health",
			"permissions"
		]
	};
	var schema63 = {
		"type": "string",
		"enum": [
			"CONNECTING",
			"REVIEW_REQUIRED",
			"CONNECTED",
			"FAILED",
			"DISCONNECTED"
		]
	};
	var schema69 = {
		"type": "object",
		"properties": {
			"acknowledged": { "type": "boolean" },
			"detected": {
				"type": "array",
				"items": { "type": "string" }
			},
			"forbidden": {
				"type": "array",
				"items": { "type": "string" }
			},
			"ipAllowList": {
				"type": ["array", "null"],
				"items": { "type": "string" }
			},
			"ipAllowListStatus": { "type": "string" },
			"scope": {
				"type": "string",
				"enum": ["VERIFIED", "UNVERIFIED"]
			},
			"unsupported": {
				"type": "array",
				"items": { "type": "string" }
			}
		},
		"additionalProperties": false,
		"required": [
			"scope",
			"detected",
			"forbidden",
			"unsupported",
			"acknowledged",
			"ipAllowListStatus"
		]
	};
	var func31 = Object.prototype.hasOwnProperty;
	var func1 = require_ucs2length().default;
	var schema64 = {
		"type": "object",
		"properties": {
			"accountType": { "type": "string" },
			"balances": {
				"type": "array",
				"items": { "$ref": "#/$defs/Balance" }
			},
			"capabilities": {
				"type": "array",
				"items": { "type": "string" }
			},
			"currency": { "type": ["string", "null"] },
			"limitations": {
				"type": "array",
				"items": { "type": "string" }
			},
			"openOrders": {
				"type": "array",
				"items": { "$ref": "#/$defs/OpenOrder" }
			},
			"positions": {
				"type": "array",
				"items": { "$ref": "#/$defs/Position" }
			},
			"remoteAccountId": { "type": "string" }
		},
		"additionalProperties": false,
		"required": [
			"remoteAccountId",
			"accountType",
			"balances",
			"positions",
			"openOrders",
			"capabilities",
			"limitations"
		]
	};
	var schema65 = {
		"type": "object",
		"properties": {
			"asset": { "type": "string" },
			"available": { "type": "string" },
			"inPies": { "type": ["string", "null"] },
			"locked": { "type": ["string", "null"] },
			"reserved": { "type": ["string", "null"] },
			"restrictedAvailable": { "type": ["string", "null"] },
			"total": { "type": ["string", "null"] }
		},
		"additionalProperties": false,
		"required": ["asset", "available"]
	};
	var schema66 = {
		"type": "object",
		"properties": {
			"brokerOrderId": { "type": "string" },
			"currency": { "type": ["string", "null"] },
			"filledQuantity": { "type": ["string", "null"] },
			"filledValue": { "type": ["string", "null"] },
			"kind": {
				"type": ["string", "null"],
				"enum": [
					"NORMAL",
					"TPSL",
					"PLAN",
					null
				]
			},
			"limitPrice": { "type": ["string", "null"] },
			"notional": { "type": ["string", "null"] },
			"quantity": { "type": ["string", "null"] },
			"side": { "type": "string" },
			"status": { "type": "string" },
			"symbol": { "type": "string" },
			"triggerPrice": { "type": ["string", "null"] }
		},
		"additionalProperties": false,
		"required": [
			"brokerOrderId",
			"symbol",
			"side",
			"status"
		]
	};
	var schema67 = {
		"type": "object",
		"properties": {
			"averageEntryPrice": { "type": ["string", "null"] },
			"instrumentCurrency": { "type": ["string", "null"] },
			"marketValue": { "type": ["string", "null"] },
			"marketValueCurrency": { "type": ["string", "null"] },
			"quantity": { "type": "string" },
			"symbol": { "type": "string" }
		},
		"additionalProperties": false,
		"required": ["symbol", "quantity"]
	};
	function validate40(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate40.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.remoteAccountId === void 0 && (missing0 = "remoteAccountId") || data.accountType === void 0 && (missing0 = "accountType") || data.balances === void 0 && (missing0 = "balances") || data.positions === void 0 && (missing0 = "positions") || data.openOrders === void 0 && (missing0 = "openOrders") || data.capabilities === void 0 && (missing0 = "capabilities") || data.limitations === void 0 && (missing0 = "limitations")) {
				validate40.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "accountType" || key0 === "balances" || key0 === "capabilities" || key0 === "currency" || key0 === "limitations" || key0 === "openOrders" || key0 === "positions" || key0 === "remoteAccountId")) {
					validate40.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.accountType !== void 0) {
					if (typeof data.accountType !== "string") {
						validate40.errors = [{
							instancePath: instancePath + "/accountType",
							schemaPath: "#/properties/accountType/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.balances !== void 0) {
						let data1 = data.balances;
						if (Array.isArray(data1)) {
							const len0 = data1.length;
							for (let i0 = 0; i0 < len0; i0++) {
								let data2 = data1[i0];
								if (data2 && typeof data2 == "object" && !Array.isArray(data2)) {
									let missing1;
									if (data2.asset === void 0 && (missing1 = "asset") || data2.available === void 0 && (missing1 = "available")) {
										validate40.errors = [{
											instancePath: instancePath + "/balances/" + i0,
											schemaPath: "#/$defs/Balance/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "asset" || key1 === "available" || key1 === "inPies" || key1 === "locked" || key1 === "reserved" || key1 === "restrictedAvailable" || key1 === "total")) {
											validate40.errors = [{
												instancePath: instancePath + "/balances/" + i0,
												schemaPath: "#/$defs/Balance/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data2.asset !== void 0) {
											if (typeof data2.asset !== "string") {
												validate40.errors = [{
													instancePath: instancePath + "/balances/" + i0 + "/asset",
													schemaPath: "#/$defs/Balance/properties/asset/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data2.available !== void 0) {
												if (typeof data2.available !== "string") {
													validate40.errors = [{
														instancePath: instancePath + "/balances/" + i0 + "/available",
														schemaPath: "#/$defs/Balance/properties/available/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid3 = true;
											} else var valid3 = true;
											if (valid3) {
												if (data2.inPies !== void 0) {
													let data5 = data2.inPies;
													if (typeof data5 !== "string" && data5 !== null) {
														validate40.errors = [{
															instancePath: instancePath + "/balances/" + i0 + "/inPies",
															schemaPath: "#/$defs/Balance/properties/inPies/type",
															keyword: "type",
															params: { type: schema65.properties.inPies.type },
															message: "must be string,null"
														}];
														return false;
													}
													var valid3 = true;
												} else var valid3 = true;
												if (valid3) {
													if (data2.locked !== void 0) {
														let data6 = data2.locked;
														if (typeof data6 !== "string" && data6 !== null) {
															validate40.errors = [{
																instancePath: instancePath + "/balances/" + i0 + "/locked",
																schemaPath: "#/$defs/Balance/properties/locked/type",
																keyword: "type",
																params: { type: schema65.properties.locked.type },
																message: "must be string,null"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data2.reserved !== void 0) {
															let data7 = data2.reserved;
															if (typeof data7 !== "string" && data7 !== null) {
																validate40.errors = [{
																	instancePath: instancePath + "/balances/" + i0 + "/reserved",
																	schemaPath: "#/$defs/Balance/properties/reserved/type",
																	keyword: "type",
																	params: { type: schema65.properties.reserved.type },
																	message: "must be string,null"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
														if (valid3) {
															if (data2.restrictedAvailable !== void 0) {
																let data8 = data2.restrictedAvailable;
																if (typeof data8 !== "string" && data8 !== null) {
																	validate40.errors = [{
																		instancePath: instancePath + "/balances/" + i0 + "/restrictedAvailable",
																		schemaPath: "#/$defs/Balance/properties/restrictedAvailable/type",
																		keyword: "type",
																		params: { type: schema65.properties.restrictedAvailable.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid3 = true;
															} else var valid3 = true;
															if (valid3) {
																if (data2.total !== void 0) {
																	let data9 = data2.total;
																	if (typeof data9 !== "string" && data9 !== null) {
																		validate40.errors = [{
																			instancePath: instancePath + "/balances/" + i0 + "/total",
																			schemaPath: "#/$defs/Balance/properties/total/type",
																			keyword: "type",
																			params: { type: schema65.properties.total.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid3 = true;
																} else var valid3 = true;
															}
														}
													}
												}
											}
										}
									}
								} else {
									validate40.errors = [{
										instancePath: instancePath + "/balances/" + i0,
										schemaPath: "#/$defs/Balance/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
							}
						} else {
							validate40.errors = [{
								instancePath: instancePath + "/balances",
								schemaPath: "#/properties/balances/type",
								keyword: "type",
								params: { type: "array" },
								message: "must be array"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.capabilities !== void 0) {
							let data10 = data.capabilities;
							if (Array.isArray(data10)) {
								const len1 = data10.length;
								for (let i1 = 0; i1 < len1; i1++) if (typeof data10[i1] !== "string") {
									validate40.errors = [{
										instancePath: instancePath + "/capabilities/" + i1,
										schemaPath: "#/properties/capabilities/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate40.errors = [{
									instancePath: instancePath + "/capabilities",
									schemaPath: "#/properties/capabilities/type",
									keyword: "type",
									params: { type: "array" },
									message: "must be array"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.currency !== void 0) {
								let data12 = data.currency;
								if (typeof data12 !== "string" && data12 !== null) {
									validate40.errors = [{
										instancePath: instancePath + "/currency",
										schemaPath: "#/properties/currency/type",
										keyword: "type",
										params: { type: schema64.properties.currency.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.limitations !== void 0) {
									let data13 = data.limitations;
									if (Array.isArray(data13)) {
										const len2 = data13.length;
										for (let i2 = 0; i2 < len2; i2++) if (typeof data13[i2] !== "string") {
											validate40.errors = [{
												instancePath: instancePath + "/limitations/" + i2,
												schemaPath: "#/properties/limitations/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate40.errors = [{
											instancePath: instancePath + "/limitations",
											schemaPath: "#/properties/limitations/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.openOrders !== void 0) {
										let data15 = data.openOrders;
										if (Array.isArray(data15)) {
											const len3 = data15.length;
											for (let i3 = 0; i3 < len3; i3++) {
												let data16 = data15[i3];
												if (data16 && typeof data16 == "object" && !Array.isArray(data16)) {
													let missing2;
													if (data16.brokerOrderId === void 0 && (missing2 = "brokerOrderId") || data16.symbol === void 0 && (missing2 = "symbol") || data16.side === void 0 && (missing2 = "side") || data16.status === void 0 && (missing2 = "status")) {
														validate40.errors = [{
															instancePath: instancePath + "/openOrders/" + i3,
															schemaPath: "#/$defs/OpenOrder/required",
															keyword: "required",
															params: { missingProperty: missing2 },
															message: "must have required property '" + missing2 + "'"
														}];
														return false;
													} else {
														for (const key2 in data16) if (!func31.call(schema66.properties, key2)) {
															validate40.errors = [{
																instancePath: instancePath + "/openOrders/" + i3,
																schemaPath: "#/$defs/OpenOrder/additionalProperties",
																keyword: "additionalProperties",
																params: { additionalProperty: key2 },
																message: "must NOT have additional properties"
															}];
															return false;
														}
														if (data16.brokerOrderId !== void 0) {
															if (typeof data16.brokerOrderId !== "string") {
																validate40.errors = [{
																	instancePath: instancePath + "/openOrders/" + i3 + "/brokerOrderId",
																	schemaPath: "#/$defs/OpenOrder/properties/brokerOrderId/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid8 = true;
														} else var valid8 = true;
														if (valid8) {
															if (data16.currency !== void 0) {
																let data18 = data16.currency;
																if (typeof data18 !== "string" && data18 !== null) {
																	validate40.errors = [{
																		instancePath: instancePath + "/openOrders/" + i3 + "/currency",
																		schemaPath: "#/$defs/OpenOrder/properties/currency/type",
																		keyword: "type",
																		params: { type: schema66.properties.currency.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid8 = true;
															} else var valid8 = true;
															if (valid8) {
																if (data16.filledQuantity !== void 0) {
																	let data19 = data16.filledQuantity;
																	if (typeof data19 !== "string" && data19 !== null) {
																		validate40.errors = [{
																			instancePath: instancePath + "/openOrders/" + i3 + "/filledQuantity",
																			schemaPath: "#/$defs/OpenOrder/properties/filledQuantity/type",
																			keyword: "type",
																			params: { type: schema66.properties.filledQuantity.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid8 = true;
																} else var valid8 = true;
																if (valid8) {
																	if (data16.filledValue !== void 0) {
																		let data20 = data16.filledValue;
																		if (typeof data20 !== "string" && data20 !== null) {
																			validate40.errors = [{
																				instancePath: instancePath + "/openOrders/" + i3 + "/filledValue",
																				schemaPath: "#/$defs/OpenOrder/properties/filledValue/type",
																				keyword: "type",
																				params: { type: schema66.properties.filledValue.type },
																				message: "must be string,null"
																			}];
																			return false;
																		}
																		var valid8 = true;
																	} else var valid8 = true;
																	if (valid8) {
																		if (data16.kind !== void 0) {
																			let data21 = data16.kind;
																			if (typeof data21 !== "string" && data21 !== null) {
																				validate40.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/type",
																					keyword: "type",
																					params: { type: schema66.properties.kind.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			if (!(data21 === "NORMAL" || data21 === "TPSL" || data21 === "PLAN" || data21 === null)) {
																				validate40.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/enum",
																					keyword: "enum",
																					params: { allowedValues: schema66.properties.kind.enum },
																					message: "must be equal to one of the allowed values"
																				}];
																				return false;
																			}
																			var valid8 = true;
																		} else var valid8 = true;
																		if (valid8) {
																			if (data16.limitPrice !== void 0) {
																				let data22 = data16.limitPrice;
																				if (typeof data22 !== "string" && data22 !== null) {
																					validate40.errors = [{
																						instancePath: instancePath + "/openOrders/" + i3 + "/limitPrice",
																						schemaPath: "#/$defs/OpenOrder/properties/limitPrice/type",
																						keyword: "type",
																						params: { type: schema66.properties.limitPrice.type },
																						message: "must be string,null"
																					}];
																					return false;
																				}
																				var valid8 = true;
																			} else var valid8 = true;
																			if (valid8) {
																				if (data16.notional !== void 0) {
																					let data23 = data16.notional;
																					if (typeof data23 !== "string" && data23 !== null) {
																						validate40.errors = [{
																							instancePath: instancePath + "/openOrders/" + i3 + "/notional",
																							schemaPath: "#/$defs/OpenOrder/properties/notional/type",
																							keyword: "type",
																							params: { type: schema66.properties.notional.type },
																							message: "must be string,null"
																						}];
																						return false;
																					}
																					var valid8 = true;
																				} else var valid8 = true;
																				if (valid8) {
																					if (data16.quantity !== void 0) {
																						let data24 = data16.quantity;
																						if (typeof data24 !== "string" && data24 !== null) {
																							validate40.errors = [{
																								instancePath: instancePath + "/openOrders/" + i3 + "/quantity",
																								schemaPath: "#/$defs/OpenOrder/properties/quantity/type",
																								keyword: "type",
																								params: { type: schema66.properties.quantity.type },
																								message: "must be string,null"
																							}];
																							return false;
																						}
																						var valid8 = true;
																					} else var valid8 = true;
																					if (valid8) {
																						if (data16.side !== void 0) {
																							if (typeof data16.side !== "string") {
																								validate40.errors = [{
																									instancePath: instancePath + "/openOrders/" + i3 + "/side",
																									schemaPath: "#/$defs/OpenOrder/properties/side/type",
																									keyword: "type",
																									params: { type: "string" },
																									message: "must be string"
																								}];
																								return false;
																							}
																							var valid8 = true;
																						} else var valid8 = true;
																						if (valid8) {
																							if (data16.status !== void 0) {
																								if (typeof data16.status !== "string") {
																									validate40.errors = [{
																										instancePath: instancePath + "/openOrders/" + i3 + "/status",
																										schemaPath: "#/$defs/OpenOrder/properties/status/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid8 = true;
																							} else var valid8 = true;
																							if (valid8) {
																								if (data16.symbol !== void 0) {
																									if (typeof data16.symbol !== "string") {
																										validate40.errors = [{
																											instancePath: instancePath + "/openOrders/" + i3 + "/symbol",
																											schemaPath: "#/$defs/OpenOrder/properties/symbol/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									var valid8 = true;
																								} else var valid8 = true;
																								if (valid8) {
																									if (data16.triggerPrice !== void 0) {
																										let data28 = data16.triggerPrice;
																										if (typeof data28 !== "string" && data28 !== null) {
																											validate40.errors = [{
																												instancePath: instancePath + "/openOrders/" + i3 + "/triggerPrice",
																												schemaPath: "#/$defs/OpenOrder/properties/triggerPrice/type",
																												keyword: "type",
																												params: { type: schema66.properties.triggerPrice.type },
																												message: "must be string,null"
																											}];
																											return false;
																										}
																										var valid8 = true;
																									} else var valid8 = true;
																								}
																							}
																						}
																					}
																				}
																			}
																		}
																	}
																}
															}
														}
													}
												} else {
													validate40.errors = [{
														instancePath: instancePath + "/openOrders/" + i3,
														schemaPath: "#/$defs/OpenOrder/type",
														keyword: "type",
														params: { type: "object" },
														message: "must be object"
													}];
													return false;
												}
											}
										} else {
											validate40.errors = [{
												instancePath: instancePath + "/openOrders",
												schemaPath: "#/properties/openOrders/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.positions !== void 0) {
											let data29 = data.positions;
											if (Array.isArray(data29)) {
												const len4 = data29.length;
												for (let i4 = 0; i4 < len4; i4++) {
													let data30 = data29[i4];
													if (data30 && typeof data30 == "object" && !Array.isArray(data30)) {
														let missing3;
														if (data30.symbol === void 0 && (missing3 = "symbol") || data30.quantity === void 0 && (missing3 = "quantity")) {
															validate40.errors = [{
																instancePath: instancePath + "/positions/" + i4,
																schemaPath: "#/$defs/Position/required",
																keyword: "required",
																params: { missingProperty: missing3 },
																message: "must have required property '" + missing3 + "'"
															}];
															return false;
														} else {
															for (const key3 in data30) if (!(key3 === "averageEntryPrice" || key3 === "instrumentCurrency" || key3 === "marketValue" || key3 === "marketValueCurrency" || key3 === "quantity" || key3 === "symbol")) {
																validate40.errors = [{
																	instancePath: instancePath + "/positions/" + i4,
																	schemaPath: "#/$defs/Position/additionalProperties",
																	keyword: "additionalProperties",
																	params: { additionalProperty: key3 },
																	message: "must NOT have additional properties"
																}];
																return false;
															}
															if (data30.averageEntryPrice !== void 0) {
																let data31 = data30.averageEntryPrice;
																if (typeof data31 !== "string" && data31 !== null) {
																	validate40.errors = [{
																		instancePath: instancePath + "/positions/" + i4 + "/averageEntryPrice",
																		schemaPath: "#/$defs/Position/properties/averageEntryPrice/type",
																		keyword: "type",
																		params: { type: schema67.properties.averageEntryPrice.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid11 = true;
															} else var valid11 = true;
															if (valid11) {
																if (data30.instrumentCurrency !== void 0) {
																	let data32 = data30.instrumentCurrency;
																	if (typeof data32 !== "string" && data32 !== null) {
																		validate40.errors = [{
																			instancePath: instancePath + "/positions/" + i4 + "/instrumentCurrency",
																			schemaPath: "#/$defs/Position/properties/instrumentCurrency/type",
																			keyword: "type",
																			params: { type: schema67.properties.instrumentCurrency.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid11 = true;
																} else var valid11 = true;
																if (valid11) {
																	if (data30.marketValue !== void 0) {
																		let data33 = data30.marketValue;
																		if (typeof data33 !== "string" && data33 !== null) {
																			validate40.errors = [{
																				instancePath: instancePath + "/positions/" + i4 + "/marketValue",
																				schemaPath: "#/$defs/Position/properties/marketValue/type",
																				keyword: "type",
																				params: { type: schema67.properties.marketValue.type },
																				message: "must be string,null"
																			}];
																			return false;
																		}
																		var valid11 = true;
																	} else var valid11 = true;
																	if (valid11) {
																		if (data30.marketValueCurrency !== void 0) {
																			let data34 = data30.marketValueCurrency;
																			if (typeof data34 !== "string" && data34 !== null) {
																				validate40.errors = [{
																					instancePath: instancePath + "/positions/" + i4 + "/marketValueCurrency",
																					schemaPath: "#/$defs/Position/properties/marketValueCurrency/type",
																					keyword: "type",
																					params: { type: schema67.properties.marketValueCurrency.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			var valid11 = true;
																		} else var valid11 = true;
																		if (valid11) {
																			if (data30.quantity !== void 0) {
																				if (typeof data30.quantity !== "string") {
																					validate40.errors = [{
																						instancePath: instancePath + "/positions/" + i4 + "/quantity",
																						schemaPath: "#/$defs/Position/properties/quantity/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																				var valid11 = true;
																			} else var valid11 = true;
																			if (valid11) {
																				if (data30.symbol !== void 0) {
																					if (typeof data30.symbol !== "string") {
																						validate40.errors = [{
																							instancePath: instancePath + "/positions/" + i4 + "/symbol",
																							schemaPath: "#/$defs/Position/properties/symbol/type",
																							keyword: "type",
																							params: { type: "string" },
																							message: "must be string"
																						}];
																						return false;
																					}
																					var valid11 = true;
																				} else var valid11 = true;
																			}
																		}
																	}
																}
															}
														}
													} else {
														validate40.errors = [{
															instancePath: instancePath + "/positions/" + i4,
															schemaPath: "#/$defs/Position/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														}];
														return false;
													}
												}
											} else {
												validate40.errors = [{
													instancePath: instancePath + "/positions",
													schemaPath: "#/properties/positions/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.remoteAccountId !== void 0) {
												if (typeof data.remoteAccountId !== "string") {
													validate40.errors = [{
														instancePath: instancePath + "/remoteAccountId",
														schemaPath: "#/properties/remoteAccountId/type",
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
			}
		} else {
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
	function validate82(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate82.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.connectionId === void 0 && (missing0 = "connectionId") || data.workspaceId === void 0 && (missing0 = "workspaceId") || data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment") || data.label === void 0 && (missing0 = "label") || data.createdAt === void 0 && (missing0 = "createdAt") || data.updatedAt === void 0 && (missing0 = "updatedAt") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.connectionState === void 0 && (missing0 = "connectionState") || data.health === void 0 && (missing0 = "health") || data.permissions === void 0 && (missing0 = "permissions")) {
					validate82.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema62.properties, key0)) {
						validate82.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.connectionId !== void 0) {
							let data0 = data.connectionId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate82.errors = [{
											instancePath: instancePath + "/connectionId",
											schemaPath: "#/properties/connectionId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate82.errors = [{
										instancePath: instancePath + "/connectionId",
										schemaPath: "#/properties/connectionId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.connectionState !== void 0) {
								let data1 = data.connectionState;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate82.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CONNECTING" || data1 === "REVIEW_REQUIRED" || data1 === "CONNECTED" || data1 === "FAILED" || data1 === "DISCONNECTED")) {
									validate82.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/enum",
										keyword: "enum",
										params: { allowedValues: schema63.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.createdAt !== void 0) {
									const _errs7 = errors;
									if (typeof data.createdAt !== "string") {
										validate82.errors = [{
											instancePath: instancePath + "/createdAt",
											schemaPath: "#/properties/createdAt/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.data !== void 0) {
										let data3 = data.data;
										const _errs9 = errors;
										const _errs10 = errors;
										let valid2 = false;
										const _errs11 = errors;
										if (!validate40(data3, {
											instancePath: instancePath + "/data",
											parentData: data,
											parentDataProperty: "data",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate40.errors : vErrors.concat(validate40.errors);
											errors = vErrors.length;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										const _errs12 = errors;
										if (data3 !== null) {
											const err0 = {
												instancePath: instancePath + "/data",
												schemaPath: "#/properties/data/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										var _valid0 = _errs12 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err1 = {
												instancePath: instancePath + "/data",
												schemaPath: "#/properties/data/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
											validate82.errors = vErrors;
											return false;
										} else {
											errors = _errs10;
											if (vErrors !== null) {
												if (_errs10) vErrors.length = _errs10;
												else vErrors = null;
											}
										}
										var valid0 = _errs9 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.environment !== void 0) {
											const _errs14 = errors;
											if (typeof data.environment !== "string") {
												validate82.errors = [{
													instancePath: instancePath + "/environment",
													schemaPath: "#/properties/environment/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = _errs14 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.health !== void 0) {
												let data5 = data.health;
												const _errs16 = errors;
												if (errors === errors) {
													if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
														let missing1;
														if (data5.connection === void 0 && (missing1 = "connection") || data5.authentication === void 0 && (missing1 = "authentication") || data5.credential === void 0 && (missing1 = "credential") || data5.privateStream === void 0 && (missing1 = "privateStream") || data5.reconciliation === void 0 && (missing1 = "reconciliation") || data5.executionEligibility === void 0 && (missing1 = "executionEligibility") || data5.arming === void 0 && (missing1 = "arming") || data5.reason === void 0 && (missing1 = "reason")) {
															validate82.errors = [{
																instancePath: instancePath + "/health",
																schemaPath: "#/$defs/AccountHealth/required",
																keyword: "required",
																params: { missingProperty: missing1 },
																message: "must have required property '" + missing1 + "'"
															}];
															return false;
														} else {
															const _errs19 = errors;
															for (const key1 in data5) if (!(key1 === "arming" || key1 === "authentication" || key1 === "connection" || key1 === "credential" || key1 === "executionEligibility" || key1 === "privateStream" || key1 === "reason" || key1 === "reconciliation")) {
																validate82.errors = [{
																	instancePath: instancePath + "/health",
																	schemaPath: "#/$defs/AccountHealth/additionalProperties",
																	keyword: "additionalProperties",
																	params: { additionalProperty: key1 },
																	message: "must NOT have additional properties"
																}];
																return false;
															}
															if (_errs19 === errors) {
																if (data5.arming !== void 0) {
																	const _errs20 = errors;
																	if (typeof data5.arming !== "string") {
																		validate82.errors = [{
																			instancePath: instancePath + "/health/arming",
																			schemaPath: "#/$defs/AccountHealth/properties/arming/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid4 = _errs20 === errors;
																} else var valid4 = true;
																if (valid4) {
																	if (data5.authentication !== void 0) {
																		const _errs22 = errors;
																		if (typeof data5.authentication !== "string") {
																			validate82.errors = [{
																				instancePath: instancePath + "/health/authentication",
																				schemaPath: "#/$defs/AccountHealth/properties/authentication/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																		var valid4 = _errs22 === errors;
																	} else var valid4 = true;
																	if (valid4) {
																		if (data5.connection !== void 0) {
																			const _errs24 = errors;
																			if (typeof data5.connection !== "string") {
																				validate82.errors = [{
																					instancePath: instancePath + "/health/connection",
																					schemaPath: "#/$defs/AccountHealth/properties/connection/type",
																					keyword: "type",
																					params: { type: "string" },
																					message: "must be string"
																				}];
																				return false;
																			}
																			var valid4 = _errs24 === errors;
																		} else var valid4 = true;
																		if (valid4) {
																			if (data5.credential !== void 0) {
																				const _errs26 = errors;
																				if (typeof data5.credential !== "string") {
																					validate82.errors = [{
																						instancePath: instancePath + "/health/credential",
																						schemaPath: "#/$defs/AccountHealth/properties/credential/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																				var valid4 = _errs26 === errors;
																			} else var valid4 = true;
																			if (valid4) {
																				if (data5.executionEligibility !== void 0) {
																					const _errs28 = errors;
																					if (typeof data5.executionEligibility !== "string") {
																						validate82.errors = [{
																							instancePath: instancePath + "/health/executionEligibility",
																							schemaPath: "#/$defs/AccountHealth/properties/executionEligibility/type",
																							keyword: "type",
																							params: { type: "string" },
																							message: "must be string"
																						}];
																						return false;
																					}
																					var valid4 = _errs28 === errors;
																				} else var valid4 = true;
																				if (valid4) {
																					if (data5.privateStream !== void 0) {
																						const _errs30 = errors;
																						if (typeof data5.privateStream !== "string") {
																							validate82.errors = [{
																								instancePath: instancePath + "/health/privateStream",
																								schemaPath: "#/$defs/AccountHealth/properties/privateStream/type",
																								keyword: "type",
																								params: { type: "string" },
																								message: "must be string"
																							}];
																							return false;
																						}
																						var valid4 = _errs30 === errors;
																					} else var valid4 = true;
																					if (valid4) {
																						if (data5.reason !== void 0) {
																							const _errs32 = errors;
																							if (typeof data5.reason !== "string") {
																								validate82.errors = [{
																									instancePath: instancePath + "/health/reason",
																									schemaPath: "#/$defs/AccountHealth/properties/reason/type",
																									keyword: "type",
																									params: { type: "string" },
																									message: "must be string"
																								}];
																								return false;
																							}
																							var valid4 = _errs32 === errors;
																						} else var valid4 = true;
																						if (valid4) {
																							if (data5.reconciliation !== void 0) {
																								const _errs34 = errors;
																								if (typeof data5.reconciliation !== "string") {
																									validate82.errors = [{
																										instancePath: instancePath + "/health/reconciliation",
																										schemaPath: "#/$defs/AccountHealth/properties/reconciliation/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid4 = _errs34 === errors;
																							} else var valid4 = true;
																						}
																					}
																				}
																			}
																		}
																	}
																}
															}
														}
													} else {
														validate82.errors = [{
															instancePath: instancePath + "/health",
															schemaPath: "#/$defs/AccountHealth/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														}];
														return false;
													}
												}
												var valid0 = _errs16 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.label !== void 0) {
													const _errs36 = errors;
													if (typeof data.label !== "string") {
														validate82.errors = [{
															instancePath: instancePath + "/label",
															schemaPath: "#/properties/label/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid0 = _errs36 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.lastSuccessfulSync !== void 0) {
														let data15 = data.lastSuccessfulSync;
														const _errs38 = errors;
														if (typeof data15 !== "string" && data15 !== null) {
															validate82.errors = [{
																instancePath: instancePath + "/lastSuccessfulSync",
																schemaPath: "#/properties/lastSuccessfulSync/type",
																keyword: "type",
																params: { type: schema62.properties.lastSuccessfulSync.type },
																message: "must be string,null"
															}];
															return false;
														}
														var valid0 = _errs38 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.permissions !== void 0) {
															let data16 = data.permissions;
															const _errs40 = errors;
															if (errors === errors) {
																if (data16 && typeof data16 == "object" && !Array.isArray(data16)) {
																	let missing2;
																	if (data16.scope === void 0 && (missing2 = "scope") || data16.detected === void 0 && (missing2 = "detected") || data16.forbidden === void 0 && (missing2 = "forbidden") || data16.unsupported === void 0 && (missing2 = "unsupported") || data16.acknowledged === void 0 && (missing2 = "acknowledged") || data16.ipAllowListStatus === void 0 && (missing2 = "ipAllowListStatus")) {
																		validate82.errors = [{
																			instancePath: instancePath + "/permissions",
																			schemaPath: "#/$defs/PermissionReview/required",
																			keyword: "required",
																			params: { missingProperty: missing2 },
																			message: "must have required property '" + missing2 + "'"
																		}];
																		return false;
																	} else {
																		const _errs43 = errors;
																		for (const key2 in data16) if (!(key2 === "acknowledged" || key2 === "detected" || key2 === "forbidden" || key2 === "ipAllowList" || key2 === "ipAllowListStatus" || key2 === "scope" || key2 === "unsupported")) {
																			validate82.errors = [{
																				instancePath: instancePath + "/permissions",
																				schemaPath: "#/$defs/PermissionReview/additionalProperties",
																				keyword: "additionalProperties",
																				params: { additionalProperty: key2 },
																				message: "must NOT have additional properties"
																			}];
																			return false;
																		}
																		if (_errs43 === errors) {
																			if (data16.acknowledged !== void 0) {
																				const _errs44 = errors;
																				if (typeof data16.acknowledged !== "boolean") {
																					validate82.errors = [{
																						instancePath: instancePath + "/permissions/acknowledged",
																						schemaPath: "#/$defs/PermissionReview/properties/acknowledged/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					}];
																					return false;
																				}
																				var valid6 = _errs44 === errors;
																			} else var valid6 = true;
																			if (valid6) {
																				if (data16.detected !== void 0) {
																					let data18 = data16.detected;
																					const _errs46 = errors;
																					if (errors === _errs46) {
																						if (Array.isArray(data18)) {
																							const len0 = data18.length;
																							for (let i0 = 0; i0 < len0; i0++) {
																								const _errs48 = errors;
																								if (typeof data18[i0] !== "string") {
																									validate82.errors = [{
																										instancePath: instancePath + "/permissions/detected/" + i0,
																										schemaPath: "#/$defs/PermissionReview/properties/detected/items/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								if (!(_errs48 === errors)) break;
																							}
																						} else {
																							validate82.errors = [{
																								instancePath: instancePath + "/permissions/detected",
																								schemaPath: "#/$defs/PermissionReview/properties/detected/type",
																								keyword: "type",
																								params: { type: "array" },
																								message: "must be array"
																							}];
																							return false;
																						}
																					}
																					var valid6 = _errs46 === errors;
																				} else var valid6 = true;
																				if (valid6) {
																					if (data16.forbidden !== void 0) {
																						let data20 = data16.forbidden;
																						const _errs50 = errors;
																						if (errors === _errs50) {
																							if (Array.isArray(data20)) {
																								const len1 = data20.length;
																								for (let i1 = 0; i1 < len1; i1++) {
																									const _errs52 = errors;
																									if (typeof data20[i1] !== "string") {
																										validate82.errors = [{
																											instancePath: instancePath + "/permissions/forbidden/" + i1,
																											schemaPath: "#/$defs/PermissionReview/properties/forbidden/items/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(_errs52 === errors)) break;
																								}
																							} else {
																								validate82.errors = [{
																									instancePath: instancePath + "/permissions/forbidden",
																									schemaPath: "#/$defs/PermissionReview/properties/forbidden/type",
																									keyword: "type",
																									params: { type: "array" },
																									message: "must be array"
																								}];
																								return false;
																							}
																						}
																						var valid6 = _errs50 === errors;
																					} else var valid6 = true;
																					if (valid6) {
																						if (data16.ipAllowList !== void 0) {
																							let data22 = data16.ipAllowList;
																							const _errs54 = errors;
																							if (!Array.isArray(data22) && data22 !== null) {
																								validate82.errors = [{
																									instancePath: instancePath + "/permissions/ipAllowList",
																									schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
																									keyword: "type",
																									params: { type: schema69.properties.ipAllowList.type },
																									message: "must be array,null"
																								}];
																								return false;
																							}
																							if (errors === _errs54) {
																								if (Array.isArray(data22)) {
																									const len2 = data22.length;
																									for (let i2 = 0; i2 < len2; i2++) {
																										const _errs56 = errors;
																										if (typeof data22[i2] !== "string") {
																											validate82.errors = [{
																												instancePath: instancePath + "/permissions/ipAllowList/" + i2,
																												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/items/type",
																												keyword: "type",
																												params: { type: "string" },
																												message: "must be string"
																											}];
																											return false;
																										}
																										if (!(_errs56 === errors)) break;
																									}
																								}
																							}
																							var valid6 = _errs54 === errors;
																						} else var valid6 = true;
																						if (valid6) {
																							if (data16.ipAllowListStatus !== void 0) {
																								const _errs58 = errors;
																								if (typeof data16.ipAllowListStatus !== "string") {
																									validate82.errors = [{
																										instancePath: instancePath + "/permissions/ipAllowListStatus",
																										schemaPath: "#/$defs/PermissionReview/properties/ipAllowListStatus/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid6 = _errs58 === errors;
																							} else var valid6 = true;
																							if (valid6) {
																								if (data16.scope !== void 0) {
																									let data25 = data16.scope;
																									const _errs60 = errors;
																									if (typeof data25 !== "string") {
																										validate82.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(data25 === "VERIFIED" || data25 === "UNVERIFIED")) {
																										validate82.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
																											keyword: "enum",
																											params: { allowedValues: schema69.properties.scope.enum },
																											message: "must be equal to one of the allowed values"
																										}];
																										return false;
																									}
																									var valid6 = _errs60 === errors;
																								} else var valid6 = true;
																								if (valid6) {
																									if (data16.unsupported !== void 0) {
																										let data26 = data16.unsupported;
																										const _errs62 = errors;
																										if (errors === _errs62) {
																											if (Array.isArray(data26)) {
																												const len3 = data26.length;
																												for (let i3 = 0; i3 < len3; i3++) {
																													const _errs64 = errors;
																													if (typeof data26[i3] !== "string") {
																														validate82.errors = [{
																															instancePath: instancePath + "/permissions/unsupported/" + i3,
																															schemaPath: "#/$defs/PermissionReview/properties/unsupported/items/type",
																															keyword: "type",
																															params: { type: "string" },
																															message: "must be string"
																														}];
																														return false;
																													}
																													if (!(_errs64 === errors)) break;
																												}
																											} else {
																												validate82.errors = [{
																													instancePath: instancePath + "/permissions/unsupported",
																													schemaPath: "#/$defs/PermissionReview/properties/unsupported/type",
																													keyword: "type",
																													params: { type: "array" },
																													message: "must be array"
																												}];
																												return false;
																											}
																										}
																										var valid6 = _errs62 === errors;
																									} else var valid6 = true;
																								}
																							}
																						}
																					}
																				}
																			}
																		}
																	}
																} else {
																	validate82.errors = [{
																		instancePath: instancePath + "/permissions",
																		schemaPath: "#/$defs/PermissionReview/type",
																		keyword: "type",
																		params: { type: "object" },
																		message: "must be object"
																	}];
																	return false;
																}
															}
															var valid0 = _errs40 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.providerId !== void 0) {
																const _errs66 = errors;
																if (typeof data.providerId !== "string") {
																	validate82.errors = [{
																		instancePath: instancePath + "/providerId",
																		schemaPath: "#/properties/providerId/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																var valid0 = _errs66 === errors;
															} else var valid0 = true;
															if (valid0) {
																if (data.stateVersion !== void 0) {
																	let data29 = data.stateVersion;
																	const _errs68 = errors;
																	if (errors === _errs68) {
																		if (typeof data29 === "string") {
																			if (func1(data29) < 1) {
																				validate82.errors = [{
																					instancePath: instancePath + "/stateVersion",
																					schemaPath: "#/properties/stateVersion/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate82.errors = [{
																				instancePath: instancePath + "/stateVersion",
																				schemaPath: "#/properties/stateVersion/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																	}
																	var valid0 = _errs68 === errors;
																} else var valid0 = true;
																if (valid0) {
																	if (data.updatedAt !== void 0) {
																		const _errs70 = errors;
																		if (typeof data.updatedAt !== "string") {
																			validate82.errors = [{
																				instancePath: instancePath + "/updatedAt",
																				schemaPath: "#/properties/updatedAt/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																		var valid0 = _errs70 === errors;
																	} else var valid0 = true;
																	if (valid0) {
																		if (data.workspaceId !== void 0) {
																			let data31 = data.workspaceId;
																			const _errs72 = errors;
																			if (errors === _errs72) {
																				if (typeof data31 === "string") {
																					if (func1(data31) < 1) {
																						validate82.errors = [{
																							instancePath: instancePath + "/workspaceId",
																							schemaPath: "#/properties/workspaceId/minLength",
																							keyword: "minLength",
																							params: { limit: 1 },
																							message: "must NOT have fewer than 1 characters"
																						}];
																						return false;
																					}
																				} else {
																					validate82.errors = [{
																						instancePath: instancePath + "/workspaceId",
																						schemaPath: "#/properties/workspaceId/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																			}
																			var valid0 = _errs72 === errors;
																		} else var valid0 = true;
																	}
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate82.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate82.errors = vErrors;
		return errors === 0;
	}
	validate82.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountData = validate84;
	function validate84(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate84.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.remoteAccountId === void 0 && (missing0 = "remoteAccountId") || data.accountType === void 0 && (missing0 = "accountType") || data.balances === void 0 && (missing0 = "balances") || data.positions === void 0 && (missing0 = "positions") || data.openOrders === void 0 && (missing0 = "openOrders") || data.capabilities === void 0 && (missing0 = "capabilities") || data.limitations === void 0 && (missing0 = "limitations")) {
				validate84.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "accountType" || key0 === "balances" || key0 === "capabilities" || key0 === "currency" || key0 === "limitations" || key0 === "openOrders" || key0 === "positions" || key0 === "remoteAccountId")) {
					validate84.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.accountType !== void 0) {
					if (typeof data.accountType !== "string") {
						validate84.errors = [{
							instancePath: instancePath + "/accountType",
							schemaPath: "#/properties/accountType/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.balances !== void 0) {
						let data1 = data.balances;
						if (Array.isArray(data1)) {
							const len0 = data1.length;
							for (let i0 = 0; i0 < len0; i0++) {
								let data2 = data1[i0];
								if (data2 && typeof data2 == "object" && !Array.isArray(data2)) {
									let missing1;
									if (data2.asset === void 0 && (missing1 = "asset") || data2.available === void 0 && (missing1 = "available")) {
										validate84.errors = [{
											instancePath: instancePath + "/balances/" + i0,
											schemaPath: "#/$defs/Balance/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "asset" || key1 === "available" || key1 === "inPies" || key1 === "locked" || key1 === "reserved" || key1 === "restrictedAvailable" || key1 === "total")) {
											validate84.errors = [{
												instancePath: instancePath + "/balances/" + i0,
												schemaPath: "#/$defs/Balance/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data2.asset !== void 0) {
											if (typeof data2.asset !== "string") {
												validate84.errors = [{
													instancePath: instancePath + "/balances/" + i0 + "/asset",
													schemaPath: "#/$defs/Balance/properties/asset/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data2.available !== void 0) {
												if (typeof data2.available !== "string") {
													validate84.errors = [{
														instancePath: instancePath + "/balances/" + i0 + "/available",
														schemaPath: "#/$defs/Balance/properties/available/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid3 = true;
											} else var valid3 = true;
											if (valid3) {
												if (data2.inPies !== void 0) {
													let data5 = data2.inPies;
													if (typeof data5 !== "string" && data5 !== null) {
														validate84.errors = [{
															instancePath: instancePath + "/balances/" + i0 + "/inPies",
															schemaPath: "#/$defs/Balance/properties/inPies/type",
															keyword: "type",
															params: { type: schema65.properties.inPies.type },
															message: "must be string,null"
														}];
														return false;
													}
													var valid3 = true;
												} else var valid3 = true;
												if (valid3) {
													if (data2.locked !== void 0) {
														let data6 = data2.locked;
														if (typeof data6 !== "string" && data6 !== null) {
															validate84.errors = [{
																instancePath: instancePath + "/balances/" + i0 + "/locked",
																schemaPath: "#/$defs/Balance/properties/locked/type",
																keyword: "type",
																params: { type: schema65.properties.locked.type },
																message: "must be string,null"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data2.reserved !== void 0) {
															let data7 = data2.reserved;
															if (typeof data7 !== "string" && data7 !== null) {
																validate84.errors = [{
																	instancePath: instancePath + "/balances/" + i0 + "/reserved",
																	schemaPath: "#/$defs/Balance/properties/reserved/type",
																	keyword: "type",
																	params: { type: schema65.properties.reserved.type },
																	message: "must be string,null"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
														if (valid3) {
															if (data2.restrictedAvailable !== void 0) {
																let data8 = data2.restrictedAvailable;
																if (typeof data8 !== "string" && data8 !== null) {
																	validate84.errors = [{
																		instancePath: instancePath + "/balances/" + i0 + "/restrictedAvailable",
																		schemaPath: "#/$defs/Balance/properties/restrictedAvailable/type",
																		keyword: "type",
																		params: { type: schema65.properties.restrictedAvailable.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid3 = true;
															} else var valid3 = true;
															if (valid3) {
																if (data2.total !== void 0) {
																	let data9 = data2.total;
																	if (typeof data9 !== "string" && data9 !== null) {
																		validate84.errors = [{
																			instancePath: instancePath + "/balances/" + i0 + "/total",
																			schemaPath: "#/$defs/Balance/properties/total/type",
																			keyword: "type",
																			params: { type: schema65.properties.total.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid3 = true;
																} else var valid3 = true;
															}
														}
													}
												}
											}
										}
									}
								} else {
									validate84.errors = [{
										instancePath: instancePath + "/balances/" + i0,
										schemaPath: "#/$defs/Balance/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
							}
						} else {
							validate84.errors = [{
								instancePath: instancePath + "/balances",
								schemaPath: "#/properties/balances/type",
								keyword: "type",
								params: { type: "array" },
								message: "must be array"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.capabilities !== void 0) {
							let data10 = data.capabilities;
							if (Array.isArray(data10)) {
								const len1 = data10.length;
								for (let i1 = 0; i1 < len1; i1++) if (typeof data10[i1] !== "string") {
									validate84.errors = [{
										instancePath: instancePath + "/capabilities/" + i1,
										schemaPath: "#/properties/capabilities/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate84.errors = [{
									instancePath: instancePath + "/capabilities",
									schemaPath: "#/properties/capabilities/type",
									keyword: "type",
									params: { type: "array" },
									message: "must be array"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.currency !== void 0) {
								let data12 = data.currency;
								if (typeof data12 !== "string" && data12 !== null) {
									validate84.errors = [{
										instancePath: instancePath + "/currency",
										schemaPath: "#/properties/currency/type",
										keyword: "type",
										params: { type: schema64.properties.currency.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.limitations !== void 0) {
									let data13 = data.limitations;
									if (Array.isArray(data13)) {
										const len2 = data13.length;
										for (let i2 = 0; i2 < len2; i2++) if (typeof data13[i2] !== "string") {
											validate84.errors = [{
												instancePath: instancePath + "/limitations/" + i2,
												schemaPath: "#/properties/limitations/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate84.errors = [{
											instancePath: instancePath + "/limitations",
											schemaPath: "#/properties/limitations/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.openOrders !== void 0) {
										let data15 = data.openOrders;
										if (Array.isArray(data15)) {
											const len3 = data15.length;
											for (let i3 = 0; i3 < len3; i3++) {
												let data16 = data15[i3];
												if (data16 && typeof data16 == "object" && !Array.isArray(data16)) {
													let missing2;
													if (data16.brokerOrderId === void 0 && (missing2 = "brokerOrderId") || data16.symbol === void 0 && (missing2 = "symbol") || data16.side === void 0 && (missing2 = "side") || data16.status === void 0 && (missing2 = "status")) {
														validate84.errors = [{
															instancePath: instancePath + "/openOrders/" + i3,
															schemaPath: "#/$defs/OpenOrder/required",
															keyword: "required",
															params: { missingProperty: missing2 },
															message: "must have required property '" + missing2 + "'"
														}];
														return false;
													} else {
														for (const key2 in data16) if (!func31.call(schema66.properties, key2)) {
															validate84.errors = [{
																instancePath: instancePath + "/openOrders/" + i3,
																schemaPath: "#/$defs/OpenOrder/additionalProperties",
																keyword: "additionalProperties",
																params: { additionalProperty: key2 },
																message: "must NOT have additional properties"
															}];
															return false;
														}
														if (data16.brokerOrderId !== void 0) {
															if (typeof data16.brokerOrderId !== "string") {
																validate84.errors = [{
																	instancePath: instancePath + "/openOrders/" + i3 + "/brokerOrderId",
																	schemaPath: "#/$defs/OpenOrder/properties/brokerOrderId/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid8 = true;
														} else var valid8 = true;
														if (valid8) {
															if (data16.currency !== void 0) {
																let data18 = data16.currency;
																if (typeof data18 !== "string" && data18 !== null) {
																	validate84.errors = [{
																		instancePath: instancePath + "/openOrders/" + i3 + "/currency",
																		schemaPath: "#/$defs/OpenOrder/properties/currency/type",
																		keyword: "type",
																		params: { type: schema66.properties.currency.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid8 = true;
															} else var valid8 = true;
															if (valid8) {
																if (data16.filledQuantity !== void 0) {
																	let data19 = data16.filledQuantity;
																	if (typeof data19 !== "string" && data19 !== null) {
																		validate84.errors = [{
																			instancePath: instancePath + "/openOrders/" + i3 + "/filledQuantity",
																			schemaPath: "#/$defs/OpenOrder/properties/filledQuantity/type",
																			keyword: "type",
																			params: { type: schema66.properties.filledQuantity.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid8 = true;
																} else var valid8 = true;
																if (valid8) {
																	if (data16.filledValue !== void 0) {
																		let data20 = data16.filledValue;
																		if (typeof data20 !== "string" && data20 !== null) {
																			validate84.errors = [{
																				instancePath: instancePath + "/openOrders/" + i3 + "/filledValue",
																				schemaPath: "#/$defs/OpenOrder/properties/filledValue/type",
																				keyword: "type",
																				params: { type: schema66.properties.filledValue.type },
																				message: "must be string,null"
																			}];
																			return false;
																		}
																		var valid8 = true;
																	} else var valid8 = true;
																	if (valid8) {
																		if (data16.kind !== void 0) {
																			let data21 = data16.kind;
																			if (typeof data21 !== "string" && data21 !== null) {
																				validate84.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/type",
																					keyword: "type",
																					params: { type: schema66.properties.kind.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			if (!(data21 === "NORMAL" || data21 === "TPSL" || data21 === "PLAN" || data21 === null)) {
																				validate84.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/enum",
																					keyword: "enum",
																					params: { allowedValues: schema66.properties.kind.enum },
																					message: "must be equal to one of the allowed values"
																				}];
																				return false;
																			}
																			var valid8 = true;
																		} else var valid8 = true;
																		if (valid8) {
																			if (data16.limitPrice !== void 0) {
																				let data22 = data16.limitPrice;
																				if (typeof data22 !== "string" && data22 !== null) {
																					validate84.errors = [{
																						instancePath: instancePath + "/openOrders/" + i3 + "/limitPrice",
																						schemaPath: "#/$defs/OpenOrder/properties/limitPrice/type",
																						keyword: "type",
																						params: { type: schema66.properties.limitPrice.type },
																						message: "must be string,null"
																					}];
																					return false;
																				}
																				var valid8 = true;
																			} else var valid8 = true;
																			if (valid8) {
																				if (data16.notional !== void 0) {
																					let data23 = data16.notional;
																					if (typeof data23 !== "string" && data23 !== null) {
																						validate84.errors = [{
																							instancePath: instancePath + "/openOrders/" + i3 + "/notional",
																							schemaPath: "#/$defs/OpenOrder/properties/notional/type",
																							keyword: "type",
																							params: { type: schema66.properties.notional.type },
																							message: "must be string,null"
																						}];
																						return false;
																					}
																					var valid8 = true;
																				} else var valid8 = true;
																				if (valid8) {
																					if (data16.quantity !== void 0) {
																						let data24 = data16.quantity;
																						if (typeof data24 !== "string" && data24 !== null) {
																							validate84.errors = [{
																								instancePath: instancePath + "/openOrders/" + i3 + "/quantity",
																								schemaPath: "#/$defs/OpenOrder/properties/quantity/type",
																								keyword: "type",
																								params: { type: schema66.properties.quantity.type },
																								message: "must be string,null"
																							}];
																							return false;
																						}
																						var valid8 = true;
																					} else var valid8 = true;
																					if (valid8) {
																						if (data16.side !== void 0) {
																							if (typeof data16.side !== "string") {
																								validate84.errors = [{
																									instancePath: instancePath + "/openOrders/" + i3 + "/side",
																									schemaPath: "#/$defs/OpenOrder/properties/side/type",
																									keyword: "type",
																									params: { type: "string" },
																									message: "must be string"
																								}];
																								return false;
																							}
																							var valid8 = true;
																						} else var valid8 = true;
																						if (valid8) {
																							if (data16.status !== void 0) {
																								if (typeof data16.status !== "string") {
																									validate84.errors = [{
																										instancePath: instancePath + "/openOrders/" + i3 + "/status",
																										schemaPath: "#/$defs/OpenOrder/properties/status/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid8 = true;
																							} else var valid8 = true;
																							if (valid8) {
																								if (data16.symbol !== void 0) {
																									if (typeof data16.symbol !== "string") {
																										validate84.errors = [{
																											instancePath: instancePath + "/openOrders/" + i3 + "/symbol",
																											schemaPath: "#/$defs/OpenOrder/properties/symbol/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									var valid8 = true;
																								} else var valid8 = true;
																								if (valid8) {
																									if (data16.triggerPrice !== void 0) {
																										let data28 = data16.triggerPrice;
																										if (typeof data28 !== "string" && data28 !== null) {
																											validate84.errors = [{
																												instancePath: instancePath + "/openOrders/" + i3 + "/triggerPrice",
																												schemaPath: "#/$defs/OpenOrder/properties/triggerPrice/type",
																												keyword: "type",
																												params: { type: schema66.properties.triggerPrice.type },
																												message: "must be string,null"
																											}];
																											return false;
																										}
																										var valid8 = true;
																									} else var valid8 = true;
																								}
																							}
																						}
																					}
																				}
																			}
																		}
																	}
																}
															}
														}
													}
												} else {
													validate84.errors = [{
														instancePath: instancePath + "/openOrders/" + i3,
														schemaPath: "#/$defs/OpenOrder/type",
														keyword: "type",
														params: { type: "object" },
														message: "must be object"
													}];
													return false;
												}
											}
										} else {
											validate84.errors = [{
												instancePath: instancePath + "/openOrders",
												schemaPath: "#/properties/openOrders/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.positions !== void 0) {
											let data29 = data.positions;
											if (Array.isArray(data29)) {
												const len4 = data29.length;
												for (let i4 = 0; i4 < len4; i4++) {
													let data30 = data29[i4];
													if (data30 && typeof data30 == "object" && !Array.isArray(data30)) {
														let missing3;
														if (data30.symbol === void 0 && (missing3 = "symbol") || data30.quantity === void 0 && (missing3 = "quantity")) {
															validate84.errors = [{
																instancePath: instancePath + "/positions/" + i4,
																schemaPath: "#/$defs/Position/required",
																keyword: "required",
																params: { missingProperty: missing3 },
																message: "must have required property '" + missing3 + "'"
															}];
															return false;
														} else {
															for (const key3 in data30) if (!(key3 === "averageEntryPrice" || key3 === "instrumentCurrency" || key3 === "marketValue" || key3 === "marketValueCurrency" || key3 === "quantity" || key3 === "symbol")) {
																validate84.errors = [{
																	instancePath: instancePath + "/positions/" + i4,
																	schemaPath: "#/$defs/Position/additionalProperties",
																	keyword: "additionalProperties",
																	params: { additionalProperty: key3 },
																	message: "must NOT have additional properties"
																}];
																return false;
															}
															if (data30.averageEntryPrice !== void 0) {
																let data31 = data30.averageEntryPrice;
																if (typeof data31 !== "string" && data31 !== null) {
																	validate84.errors = [{
																		instancePath: instancePath + "/positions/" + i4 + "/averageEntryPrice",
																		schemaPath: "#/$defs/Position/properties/averageEntryPrice/type",
																		keyword: "type",
																		params: { type: schema67.properties.averageEntryPrice.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																var valid11 = true;
															} else var valid11 = true;
															if (valid11) {
																if (data30.instrumentCurrency !== void 0) {
																	let data32 = data30.instrumentCurrency;
																	if (typeof data32 !== "string" && data32 !== null) {
																		validate84.errors = [{
																			instancePath: instancePath + "/positions/" + i4 + "/instrumentCurrency",
																			schemaPath: "#/$defs/Position/properties/instrumentCurrency/type",
																			keyword: "type",
																			params: { type: schema67.properties.instrumentCurrency.type },
																			message: "must be string,null"
																		}];
																		return false;
																	}
																	var valid11 = true;
																} else var valid11 = true;
																if (valid11) {
																	if (data30.marketValue !== void 0) {
																		let data33 = data30.marketValue;
																		if (typeof data33 !== "string" && data33 !== null) {
																			validate84.errors = [{
																				instancePath: instancePath + "/positions/" + i4 + "/marketValue",
																				schemaPath: "#/$defs/Position/properties/marketValue/type",
																				keyword: "type",
																				params: { type: schema67.properties.marketValue.type },
																				message: "must be string,null"
																			}];
																			return false;
																		}
																		var valid11 = true;
																	} else var valid11 = true;
																	if (valid11) {
																		if (data30.marketValueCurrency !== void 0) {
																			let data34 = data30.marketValueCurrency;
																			if (typeof data34 !== "string" && data34 !== null) {
																				validate84.errors = [{
																					instancePath: instancePath + "/positions/" + i4 + "/marketValueCurrency",
																					schemaPath: "#/$defs/Position/properties/marketValueCurrency/type",
																					keyword: "type",
																					params: { type: schema67.properties.marketValueCurrency.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			var valid11 = true;
																		} else var valid11 = true;
																		if (valid11) {
																			if (data30.quantity !== void 0) {
																				if (typeof data30.quantity !== "string") {
																					validate84.errors = [{
																						instancePath: instancePath + "/positions/" + i4 + "/quantity",
																						schemaPath: "#/$defs/Position/properties/quantity/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																				var valid11 = true;
																			} else var valid11 = true;
																			if (valid11) {
																				if (data30.symbol !== void 0) {
																					if (typeof data30.symbol !== "string") {
																						validate84.errors = [{
																							instancePath: instancePath + "/positions/" + i4 + "/symbol",
																							schemaPath: "#/$defs/Position/properties/symbol/type",
																							keyword: "type",
																							params: { type: "string" },
																							message: "must be string"
																						}];
																						return false;
																					}
																					var valid11 = true;
																				} else var valid11 = true;
																			}
																		}
																	}
																}
															}
														}
													} else {
														validate84.errors = [{
															instancePath: instancePath + "/positions/" + i4,
															schemaPath: "#/$defs/Position/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														}];
														return false;
													}
												}
											} else {
												validate84.errors = [{
													instancePath: instancePath + "/positions",
													schemaPath: "#/properties/positions/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.remoteAccountId !== void 0) {
												if (typeof data.remoteAccountId !== "string") {
													validate84.errors = [{
														instancePath: instancePath + "/remoteAccountId",
														schemaPath: "#/properties/remoteAccountId/type",
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
			}
		} else {
			validate84.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate84.errors = vErrors;
		return true;
	}
	validate84.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountHealth = validate85;
	function validate85(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate85.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.connection === void 0 && (missing0 = "connection") || data.authentication === void 0 && (missing0 = "authentication") || data.credential === void 0 && (missing0 = "credential") || data.privateStream === void 0 && (missing0 = "privateStream") || data.reconciliation === void 0 && (missing0 = "reconciliation") || data.executionEligibility === void 0 && (missing0 = "executionEligibility") || data.arming === void 0 && (missing0 = "arming") || data.reason === void 0 && (missing0 = "reason")) {
				validate85.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "arming" || key0 === "authentication" || key0 === "connection" || key0 === "credential" || key0 === "executionEligibility" || key0 === "privateStream" || key0 === "reason" || key0 === "reconciliation")) {
					validate85.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.arming !== void 0) {
					if (typeof data.arming !== "string") {
						validate85.errors = [{
							instancePath: instancePath + "/arming",
							schemaPath: "#/properties/arming/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.authentication !== void 0) {
						if (typeof data.authentication !== "string") {
							validate85.errors = [{
								instancePath: instancePath + "/authentication",
								schemaPath: "#/properties/authentication/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.connection !== void 0) {
							if (typeof data.connection !== "string") {
								validate85.errors = [{
									instancePath: instancePath + "/connection",
									schemaPath: "#/properties/connection/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.credential !== void 0) {
								if (typeof data.credential !== "string") {
									validate85.errors = [{
										instancePath: instancePath + "/credential",
										schemaPath: "#/properties/credential/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.executionEligibility !== void 0) {
									if (typeof data.executionEligibility !== "string") {
										validate85.errors = [{
											instancePath: instancePath + "/executionEligibility",
											schemaPath: "#/properties/executionEligibility/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.privateStream !== void 0) {
										if (typeof data.privateStream !== "string") {
											validate85.errors = [{
												instancePath: instancePath + "/privateStream",
												schemaPath: "#/properties/privateStream/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.reason !== void 0) {
											if (typeof data.reason !== "string") {
												validate85.errors = [{
													instancePath: instancePath + "/reason",
													schemaPath: "#/properties/reason/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.reconciliation !== void 0) {
												if (typeof data.reconciliation !== "string") {
													validate85.errors = [{
														instancePath: instancePath + "/reconciliation",
														schemaPath: "#/properties/reconciliation/type",
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
			}
		} else {
			validate85.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate85.errors = vErrors;
		return true;
	}
	validate85.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountMutation = validate86;
	function validate86(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate86.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.connectionId === void 0 && (missing0 = "connectionId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion")) {
				validate86.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "connectionId" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate86.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.connectionId !== void 0) {
					let data0 = data.connectionId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate86.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate86.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate86.errors = [{
							instancePath: instancePath + "/connectionId",
							schemaPath: "#/properties/connectionId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.expectedStateVersion !== void 0) {
						let data1 = data.expectedStateVersion;
						if (typeof data1 === "string") {
							if (func1(data1) > 256) {
								validate86.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate86.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate86.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data2 = data.workspaceId;
							if (typeof data2 === "string") {
								if (func1(data2) > 128) {
									validate86.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate86.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate86.errors = [{
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
		} else {
			validate86.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate86.errors = vErrors;
		return true;
	}
	validate86.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountQuery = validate87;
	function validate87(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate87.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.connectionId === void 0 && (missing0 = "connectionId")) {
				validate87.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "connectionId" || key0 === "workspaceId")) {
					validate87.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.connectionId !== void 0) {
					let data0 = data.connectionId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate87.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate87.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate87.errors = [{
							instancePath: instancePath + "/connectionId",
							schemaPath: "#/properties/connectionId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.workspaceId !== void 0) {
						let data1 = data.workspaceId;
						if (typeof data1 === "string") {
							if (func1(data1) > 128) {
								validate87.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate87.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate87.errors = [{
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
		} else {
			validate87.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate87.errors = vErrors;
		return true;
	}
	validate87.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Accounts = validate88;
	function validate39(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate39.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.connectionId === void 0 && (missing0 = "connectionId") || data.workspaceId === void 0 && (missing0 = "workspaceId") || data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment") || data.label === void 0 && (missing0 = "label") || data.createdAt === void 0 && (missing0 = "createdAt") || data.updatedAt === void 0 && (missing0 = "updatedAt") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.connectionState === void 0 && (missing0 = "connectionState") || data.health === void 0 && (missing0 = "health") || data.permissions === void 0 && (missing0 = "permissions")) {
					validate39.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema62.properties, key0)) {
						validate39.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.connectionId !== void 0) {
							let data0 = data.connectionId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate39.errors = [{
											instancePath: instancePath + "/connectionId",
											schemaPath: "#/properties/connectionId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate39.errors = [{
										instancePath: instancePath + "/connectionId",
										schemaPath: "#/properties/connectionId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.connectionState !== void 0) {
								let data1 = data.connectionState;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate39.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CONNECTING" || data1 === "REVIEW_REQUIRED" || data1 === "CONNECTED" || data1 === "FAILED" || data1 === "DISCONNECTED")) {
									validate39.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/enum",
										keyword: "enum",
										params: { allowedValues: schema63.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.createdAt !== void 0) {
									const _errs7 = errors;
									if (typeof data.createdAt !== "string") {
										validate39.errors = [{
											instancePath: instancePath + "/createdAt",
											schemaPath: "#/properties/createdAt/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.data !== void 0) {
										let data3 = data.data;
										const _errs9 = errors;
										const _errs10 = errors;
										let valid2 = false;
										const _errs11 = errors;
										if (!validate40(data3, {
											instancePath: instancePath + "/data",
											parentData: data,
											parentDataProperty: "data",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate40.errors : vErrors.concat(validate40.errors);
											errors = vErrors.length;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										const _errs12 = errors;
										if (data3 !== null) {
											const err0 = {
												instancePath: instancePath + "/data",
												schemaPath: "#/properties/data/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										var _valid0 = _errs12 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err1 = {
												instancePath: instancePath + "/data",
												schemaPath: "#/properties/data/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
											validate39.errors = vErrors;
											return false;
										} else {
											errors = _errs10;
											if (vErrors !== null) {
												if (_errs10) vErrors.length = _errs10;
												else vErrors = null;
											}
										}
										var valid0 = _errs9 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.environment !== void 0) {
											const _errs14 = errors;
											if (typeof data.environment !== "string") {
												validate39.errors = [{
													instancePath: instancePath + "/environment",
													schemaPath: "#/properties/environment/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = _errs14 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.health !== void 0) {
												let data5 = data.health;
												const _errs16 = errors;
												if (errors === errors) {
													if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
														let missing1;
														if (data5.connection === void 0 && (missing1 = "connection") || data5.authentication === void 0 && (missing1 = "authentication") || data5.credential === void 0 && (missing1 = "credential") || data5.privateStream === void 0 && (missing1 = "privateStream") || data5.reconciliation === void 0 && (missing1 = "reconciliation") || data5.executionEligibility === void 0 && (missing1 = "executionEligibility") || data5.arming === void 0 && (missing1 = "arming") || data5.reason === void 0 && (missing1 = "reason")) {
															validate39.errors = [{
																instancePath: instancePath + "/health",
																schemaPath: "#/$defs/AccountHealth/required",
																keyword: "required",
																params: { missingProperty: missing1 },
																message: "must have required property '" + missing1 + "'"
															}];
															return false;
														} else {
															const _errs19 = errors;
															for (const key1 in data5) if (!(key1 === "arming" || key1 === "authentication" || key1 === "connection" || key1 === "credential" || key1 === "executionEligibility" || key1 === "privateStream" || key1 === "reason" || key1 === "reconciliation")) {
																validate39.errors = [{
																	instancePath: instancePath + "/health",
																	schemaPath: "#/$defs/AccountHealth/additionalProperties",
																	keyword: "additionalProperties",
																	params: { additionalProperty: key1 },
																	message: "must NOT have additional properties"
																}];
																return false;
															}
															if (_errs19 === errors) {
																if (data5.arming !== void 0) {
																	const _errs20 = errors;
																	if (typeof data5.arming !== "string") {
																		validate39.errors = [{
																			instancePath: instancePath + "/health/arming",
																			schemaPath: "#/$defs/AccountHealth/properties/arming/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid4 = _errs20 === errors;
																} else var valid4 = true;
																if (valid4) {
																	if (data5.authentication !== void 0) {
																		const _errs22 = errors;
																		if (typeof data5.authentication !== "string") {
																			validate39.errors = [{
																				instancePath: instancePath + "/health/authentication",
																				schemaPath: "#/$defs/AccountHealth/properties/authentication/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																		var valid4 = _errs22 === errors;
																	} else var valid4 = true;
																	if (valid4) {
																		if (data5.connection !== void 0) {
																			const _errs24 = errors;
																			if (typeof data5.connection !== "string") {
																				validate39.errors = [{
																					instancePath: instancePath + "/health/connection",
																					schemaPath: "#/$defs/AccountHealth/properties/connection/type",
																					keyword: "type",
																					params: { type: "string" },
																					message: "must be string"
																				}];
																				return false;
																			}
																			var valid4 = _errs24 === errors;
																		} else var valid4 = true;
																		if (valid4) {
																			if (data5.credential !== void 0) {
																				const _errs26 = errors;
																				if (typeof data5.credential !== "string") {
																					validate39.errors = [{
																						instancePath: instancePath + "/health/credential",
																						schemaPath: "#/$defs/AccountHealth/properties/credential/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																				var valid4 = _errs26 === errors;
																			} else var valid4 = true;
																			if (valid4) {
																				if (data5.executionEligibility !== void 0) {
																					const _errs28 = errors;
																					if (typeof data5.executionEligibility !== "string") {
																						validate39.errors = [{
																							instancePath: instancePath + "/health/executionEligibility",
																							schemaPath: "#/$defs/AccountHealth/properties/executionEligibility/type",
																							keyword: "type",
																							params: { type: "string" },
																							message: "must be string"
																						}];
																						return false;
																					}
																					var valid4 = _errs28 === errors;
																				} else var valid4 = true;
																				if (valid4) {
																					if (data5.privateStream !== void 0) {
																						const _errs30 = errors;
																						if (typeof data5.privateStream !== "string") {
																							validate39.errors = [{
																								instancePath: instancePath + "/health/privateStream",
																								schemaPath: "#/$defs/AccountHealth/properties/privateStream/type",
																								keyword: "type",
																								params: { type: "string" },
																								message: "must be string"
																							}];
																							return false;
																						}
																						var valid4 = _errs30 === errors;
																					} else var valid4 = true;
																					if (valid4) {
																						if (data5.reason !== void 0) {
																							const _errs32 = errors;
																							if (typeof data5.reason !== "string") {
																								validate39.errors = [{
																									instancePath: instancePath + "/health/reason",
																									schemaPath: "#/$defs/AccountHealth/properties/reason/type",
																									keyword: "type",
																									params: { type: "string" },
																									message: "must be string"
																								}];
																								return false;
																							}
																							var valid4 = _errs32 === errors;
																						} else var valid4 = true;
																						if (valid4) {
																							if (data5.reconciliation !== void 0) {
																								const _errs34 = errors;
																								if (typeof data5.reconciliation !== "string") {
																									validate39.errors = [{
																										instancePath: instancePath + "/health/reconciliation",
																										schemaPath: "#/$defs/AccountHealth/properties/reconciliation/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid4 = _errs34 === errors;
																							} else var valid4 = true;
																						}
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
															instancePath: instancePath + "/health",
															schemaPath: "#/$defs/AccountHealth/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														}];
														return false;
													}
												}
												var valid0 = _errs16 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.label !== void 0) {
													const _errs36 = errors;
													if (typeof data.label !== "string") {
														validate39.errors = [{
															instancePath: instancePath + "/label",
															schemaPath: "#/properties/label/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid0 = _errs36 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.lastSuccessfulSync !== void 0) {
														let data15 = data.lastSuccessfulSync;
														const _errs38 = errors;
														if (typeof data15 !== "string" && data15 !== null) {
															validate39.errors = [{
																instancePath: instancePath + "/lastSuccessfulSync",
																schemaPath: "#/properties/lastSuccessfulSync/type",
																keyword: "type",
																params: { type: schema62.properties.lastSuccessfulSync.type },
																message: "must be string,null"
															}];
															return false;
														}
														var valid0 = _errs38 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.permissions !== void 0) {
															let data16 = data.permissions;
															const _errs40 = errors;
															if (errors === errors) {
																if (data16 && typeof data16 == "object" && !Array.isArray(data16)) {
																	let missing2;
																	if (data16.scope === void 0 && (missing2 = "scope") || data16.detected === void 0 && (missing2 = "detected") || data16.forbidden === void 0 && (missing2 = "forbidden") || data16.unsupported === void 0 && (missing2 = "unsupported") || data16.acknowledged === void 0 && (missing2 = "acknowledged") || data16.ipAllowListStatus === void 0 && (missing2 = "ipAllowListStatus")) {
																		validate39.errors = [{
																			instancePath: instancePath + "/permissions",
																			schemaPath: "#/$defs/PermissionReview/required",
																			keyword: "required",
																			params: { missingProperty: missing2 },
																			message: "must have required property '" + missing2 + "'"
																		}];
																		return false;
																	} else {
																		const _errs43 = errors;
																		for (const key2 in data16) if (!(key2 === "acknowledged" || key2 === "detected" || key2 === "forbidden" || key2 === "ipAllowList" || key2 === "ipAllowListStatus" || key2 === "scope" || key2 === "unsupported")) {
																			validate39.errors = [{
																				instancePath: instancePath + "/permissions",
																				schemaPath: "#/$defs/PermissionReview/additionalProperties",
																				keyword: "additionalProperties",
																				params: { additionalProperty: key2 },
																				message: "must NOT have additional properties"
																			}];
																			return false;
																		}
																		if (_errs43 === errors) {
																			if (data16.acknowledged !== void 0) {
																				const _errs44 = errors;
																				if (typeof data16.acknowledged !== "boolean") {
																					validate39.errors = [{
																						instancePath: instancePath + "/permissions/acknowledged",
																						schemaPath: "#/$defs/PermissionReview/properties/acknowledged/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					}];
																					return false;
																				}
																				var valid6 = _errs44 === errors;
																			} else var valid6 = true;
																			if (valid6) {
																				if (data16.detected !== void 0) {
																					let data18 = data16.detected;
																					const _errs46 = errors;
																					if (errors === _errs46) {
																						if (Array.isArray(data18)) {
																							const len0 = data18.length;
																							for (let i0 = 0; i0 < len0; i0++) {
																								const _errs48 = errors;
																								if (typeof data18[i0] !== "string") {
																									validate39.errors = [{
																										instancePath: instancePath + "/permissions/detected/" + i0,
																										schemaPath: "#/$defs/PermissionReview/properties/detected/items/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								if (!(_errs48 === errors)) break;
																							}
																						} else {
																							validate39.errors = [{
																								instancePath: instancePath + "/permissions/detected",
																								schemaPath: "#/$defs/PermissionReview/properties/detected/type",
																								keyword: "type",
																								params: { type: "array" },
																								message: "must be array"
																							}];
																							return false;
																						}
																					}
																					var valid6 = _errs46 === errors;
																				} else var valid6 = true;
																				if (valid6) {
																					if (data16.forbidden !== void 0) {
																						let data20 = data16.forbidden;
																						const _errs50 = errors;
																						if (errors === _errs50) {
																							if (Array.isArray(data20)) {
																								const len1 = data20.length;
																								for (let i1 = 0; i1 < len1; i1++) {
																									const _errs52 = errors;
																									if (typeof data20[i1] !== "string") {
																										validate39.errors = [{
																											instancePath: instancePath + "/permissions/forbidden/" + i1,
																											schemaPath: "#/$defs/PermissionReview/properties/forbidden/items/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(_errs52 === errors)) break;
																								}
																							} else {
																								validate39.errors = [{
																									instancePath: instancePath + "/permissions/forbidden",
																									schemaPath: "#/$defs/PermissionReview/properties/forbidden/type",
																									keyword: "type",
																									params: { type: "array" },
																									message: "must be array"
																								}];
																								return false;
																							}
																						}
																						var valid6 = _errs50 === errors;
																					} else var valid6 = true;
																					if (valid6) {
																						if (data16.ipAllowList !== void 0) {
																							let data22 = data16.ipAllowList;
																							const _errs54 = errors;
																							if (!Array.isArray(data22) && data22 !== null) {
																								validate39.errors = [{
																									instancePath: instancePath + "/permissions/ipAllowList",
																									schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
																									keyword: "type",
																									params: { type: schema69.properties.ipAllowList.type },
																									message: "must be array,null"
																								}];
																								return false;
																							}
																							if (errors === _errs54) {
																								if (Array.isArray(data22)) {
																									const len2 = data22.length;
																									for (let i2 = 0; i2 < len2; i2++) {
																										const _errs56 = errors;
																										if (typeof data22[i2] !== "string") {
																											validate39.errors = [{
																												instancePath: instancePath + "/permissions/ipAllowList/" + i2,
																												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/items/type",
																												keyword: "type",
																												params: { type: "string" },
																												message: "must be string"
																											}];
																											return false;
																										}
																										if (!(_errs56 === errors)) break;
																									}
																								}
																							}
																							var valid6 = _errs54 === errors;
																						} else var valid6 = true;
																						if (valid6) {
																							if (data16.ipAllowListStatus !== void 0) {
																								const _errs58 = errors;
																								if (typeof data16.ipAllowListStatus !== "string") {
																									validate39.errors = [{
																										instancePath: instancePath + "/permissions/ipAllowListStatus",
																										schemaPath: "#/$defs/PermissionReview/properties/ipAllowListStatus/type",
																										keyword: "type",
																										params: { type: "string" },
																										message: "must be string"
																									}];
																									return false;
																								}
																								var valid6 = _errs58 === errors;
																							} else var valid6 = true;
																							if (valid6) {
																								if (data16.scope !== void 0) {
																									let data25 = data16.scope;
																									const _errs60 = errors;
																									if (typeof data25 !== "string") {
																										validate39.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(data25 === "VERIFIED" || data25 === "UNVERIFIED")) {
																										validate39.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
																											keyword: "enum",
																											params: { allowedValues: schema69.properties.scope.enum },
																											message: "must be equal to one of the allowed values"
																										}];
																										return false;
																									}
																									var valid6 = _errs60 === errors;
																								} else var valid6 = true;
																								if (valid6) {
																									if (data16.unsupported !== void 0) {
																										let data26 = data16.unsupported;
																										const _errs62 = errors;
																										if (errors === _errs62) {
																											if (Array.isArray(data26)) {
																												const len3 = data26.length;
																												for (let i3 = 0; i3 < len3; i3++) {
																													const _errs64 = errors;
																													if (typeof data26[i3] !== "string") {
																														validate39.errors = [{
																															instancePath: instancePath + "/permissions/unsupported/" + i3,
																															schemaPath: "#/$defs/PermissionReview/properties/unsupported/items/type",
																															keyword: "type",
																															params: { type: "string" },
																															message: "must be string"
																														}];
																														return false;
																													}
																													if (!(_errs64 === errors)) break;
																												}
																											} else {
																												validate39.errors = [{
																													instancePath: instancePath + "/permissions/unsupported",
																													schemaPath: "#/$defs/PermissionReview/properties/unsupported/type",
																													keyword: "type",
																													params: { type: "array" },
																													message: "must be array"
																												}];
																												return false;
																											}
																										}
																										var valid6 = _errs62 === errors;
																									} else var valid6 = true;
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
																		instancePath: instancePath + "/permissions",
																		schemaPath: "#/$defs/PermissionReview/type",
																		keyword: "type",
																		params: { type: "object" },
																		message: "must be object"
																	}];
																	return false;
																}
															}
															var valid0 = _errs40 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.providerId !== void 0) {
																const _errs66 = errors;
																if (typeof data.providerId !== "string") {
																	validate39.errors = [{
																		instancePath: instancePath + "/providerId",
																		schemaPath: "#/properties/providerId/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																var valid0 = _errs66 === errors;
															} else var valid0 = true;
															if (valid0) {
																if (data.stateVersion !== void 0) {
																	let data29 = data.stateVersion;
																	const _errs68 = errors;
																	if (errors === _errs68) {
																		if (typeof data29 === "string") {
																			if (func1(data29) < 1) {
																				validate39.errors = [{
																					instancePath: instancePath + "/stateVersion",
																					schemaPath: "#/properties/stateVersion/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate39.errors = [{
																				instancePath: instancePath + "/stateVersion",
																				schemaPath: "#/properties/stateVersion/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																	}
																	var valid0 = _errs68 === errors;
																} else var valid0 = true;
																if (valid0) {
																	if (data.updatedAt !== void 0) {
																		const _errs70 = errors;
																		if (typeof data.updatedAt !== "string") {
																			validate39.errors = [{
																				instancePath: instancePath + "/updatedAt",
																				schemaPath: "#/properties/updatedAt/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}];
																			return false;
																		}
																		var valid0 = _errs70 === errors;
																	} else var valid0 = true;
																	if (valid0) {
																		if (data.workspaceId !== void 0) {
																			let data31 = data.workspaceId;
																			const _errs72 = errors;
																			if (errors === _errs72) {
																				if (typeof data31 === "string") {
																					if (func1(data31) < 1) {
																						validate39.errors = [{
																							instancePath: instancePath + "/workspaceId",
																							schemaPath: "#/properties/workspaceId/minLength",
																							keyword: "minLength",
																							params: { limit: 1 },
																							message: "must NOT have fewer than 1 characters"
																						}];
																						return false;
																					}
																				} else {
																					validate39.errors = [{
																						instancePath: instancePath + "/workspaceId",
																						schemaPath: "#/properties/workspaceId/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}];
																					return false;
																				}
																			}
																			var valid0 = _errs72 === errors;
																		} else var valid0 = true;
																	}
																}
															}
														}
													}
												}
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
		}
		validate39.errors = vErrors;
		return errors === 0;
	}
	validate39.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate88(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate88.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.accounts === void 0 && (missing0 = "accounts")) {
					validate88.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "accounts")) {
						validate88.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.accounts !== void 0) {
							let data0 = data.accounts;
							if (errors === errors) {
								if (Array.isArray(data0)) {
									const len0 = data0.length;
									for (let i0 = 0; i0 < len0; i0++) {
										const _errs4 = errors;
										if (!validate39(data0[i0], {
											instancePath: instancePath + "/accounts/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate88.errors = [{
										instancePath: instancePath + "/accounts",
										schemaPath: "#/properties/accounts/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
						}
					}
				}
			} else {
				validate88.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate88.errors = vErrors;
		return errors === 0;
	}
	validate88.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Aggregate = validate90;
	function validate90(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate90.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId")) {
				validate90.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType")) {
					validate90.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.aggregateId !== void 0) {
					let data0 = data.aggregateId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate90.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate90.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate90.errors = [{
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
						if (typeof data1 === "string") {
							if (func1(data1) > 64) {
								validate90.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/maxLength",
									keyword: "maxLength",
									params: { limit: 64 },
									message: "must NOT have more than 64 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate90.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate90.errors = [{
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
			validate90.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate90.errors = vErrors;
		return true;
	}
	validate90.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Balance = validate91;
	function validate91(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate91.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.asset === void 0 && (missing0 = "asset") || data.available === void 0 && (missing0 = "available")) {
				validate91.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "asset" || key0 === "available" || key0 === "inPies" || key0 === "locked" || key0 === "reserved" || key0 === "restrictedAvailable" || key0 === "total")) {
					validate91.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.asset !== void 0) {
					if (typeof data.asset !== "string") {
						validate91.errors = [{
							instancePath: instancePath + "/asset",
							schemaPath: "#/properties/asset/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.available !== void 0) {
						if (typeof data.available !== "string") {
							validate91.errors = [{
								instancePath: instancePath + "/available",
								schemaPath: "#/properties/available/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.inPies !== void 0) {
							let data2 = data.inPies;
							if (typeof data2 !== "string" && data2 !== null) {
								validate91.errors = [{
									instancePath: instancePath + "/inPies",
									schemaPath: "#/properties/inPies/type",
									keyword: "type",
									params: { type: schema65.properties.inPies.type },
									message: "must be string,null"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.locked !== void 0) {
								let data3 = data.locked;
								if (typeof data3 !== "string" && data3 !== null) {
									validate91.errors = [{
										instancePath: instancePath + "/locked",
										schemaPath: "#/properties/locked/type",
										keyword: "type",
										params: { type: schema65.properties.locked.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.reserved !== void 0) {
									let data4 = data.reserved;
									if (typeof data4 !== "string" && data4 !== null) {
										validate91.errors = [{
											instancePath: instancePath + "/reserved",
											schemaPath: "#/properties/reserved/type",
											keyword: "type",
											params: { type: schema65.properties.reserved.type },
											message: "must be string,null"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.restrictedAvailable !== void 0) {
										let data5 = data.restrictedAvailable;
										if (typeof data5 !== "string" && data5 !== null) {
											validate91.errors = [{
												instancePath: instancePath + "/restrictedAvailable",
												schemaPath: "#/properties/restrictedAvailable/type",
												keyword: "type",
												params: { type: schema65.properties.restrictedAvailable.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.total !== void 0) {
											let data6 = data.total;
											if (typeof data6 !== "string" && data6 !== null) {
												validate91.errors = [{
													instancePath: instancePath + "/total",
													schemaPath: "#/properties/total/type",
													keyword: "type",
													params: { type: schema65.properties.total.type },
													message: "must be string,null"
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
			validate91.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate91.errors = vErrors;
		return true;
	}
	validate91.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ChatgptLogin = validate92;
	var schema36 = {
		"type": "string",
		"enum": ["LOGIN", "RELOGIN"]
	};
	function validate92(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate92.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.action === void 0 && (missing0 = "action")) {
				validate92.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "action" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate92.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.action !== void 0) {
					let data0 = data.action;
					if (typeof data0 !== "string") {
						validate92.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/ChatgptLoginAction/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					if (!(data0 === "LOGIN" || data0 === "RELOGIN")) {
						validate92.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/ChatgptLoginAction/enum",
							keyword: "enum",
							params: { allowedValues: schema36.enum },
							message: "must be equal to one of the allowed values"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.expectedStateVersion !== void 0) {
						let data1 = data.expectedStateVersion;
						if (typeof data1 === "string") {
							if (func1(data1) > 256) {
								validate92.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate92.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate92.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data2 = data.workspaceId;
							if (typeof data2 === "string") {
								if (func1(data2) > 128) {
									validate92.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate92.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate92.errors = [{
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
		} else {
			validate92.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate92.errors = vErrors;
		return true;
	}
	validate92.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ChatgptLoginAction = validate93;
	function validate93(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate93.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate93.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "LOGIN" || data === "RELOGIN")) {
			validate93.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema36.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate93.errors = vErrors;
		return true;
	}
	validate93.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.CommandEnvelope = validate94;
	function validate94(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate94.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.command === void 0 && (missing0 = "command") || data.payload === void 0 && (missing0 = "payload")) {
				validate94.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "command" || key0 === "payload" || key0 === "requestId" || key0 === "schemaVersion")) {
					validate94.errors = [{
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
						validate94.errors = [{
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
								validate94.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate94.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate94.errors = [{
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
								validate94.errors = [{
									instancePath: instancePath + "/schemaVersion",
									schemaPath: "#/properties/schemaVersion/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								}];
								return false;
							}
							if (1 !== data2) {
								validate94.errors = [{
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
									validate94.errors = [{
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
			validate94.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate94.errors = vErrors;
		return true;
	}
	validate94.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.CompleteOnboarding = validate95;
	function validate95(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate95.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion")) {
				validate95.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate95.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.expectedStateVersion !== void 0) {
					let data0 = data.expectedStateVersion;
					if (typeof data0 === "string") {
						if (func1(data0) > 256) {
							validate95.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/maxLength",
								keyword: "maxLength",
								params: { limit: 256 },
								message: "must NOT have more than 256 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate95.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate95.errors = [{
							instancePath: instancePath + "/expectedStateVersion",
							schemaPath: "#/properties/expectedStateVersion/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.workspaceId !== void 0) {
						let data1 = data.workspaceId;
						if (typeof data1 === "string") {
							if (func1(data1) > 128) {
								validate95.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate95.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate95.errors = [{
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
		} else {
			validate95.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate95.errors = vErrors;
		return true;
	}
	validate95.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ConfigureDeepseek = validate96;
	function validate96(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate96.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion")) {
				validate96.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate96.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.expectedStateVersion !== void 0) {
					let data0 = data.expectedStateVersion;
					if (typeof data0 === "string") {
						if (func1(data0) > 256) {
							validate96.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/maxLength",
								keyword: "maxLength",
								params: { limit: 256 },
								message: "must NOT have more than 256 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate96.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate96.errors = [{
							instancePath: instancePath + "/expectedStateVersion",
							schemaPath: "#/properties/expectedStateVersion/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.workspaceId !== void 0) {
						let data1 = data.workspaceId;
						if (typeof data1 === "string") {
							if (func1(data1) > 128) {
								validate96.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate96.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate96.errors = [{
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
		} else {
			validate96.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate96.errors = vErrors;
		return true;
	}
	validate96.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Connect = validate97;
	function validate97(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate97.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (errors === _errs1) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.step === void 0 && (missing0 = "step") || data.workspaceId === void 0 && (missing0 = "workspaceId") || data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment") || data.label === void 0 && (missing0 = "label")) {
					const err0 = {
						instancePath,
						schemaPath: "#/oneOf/0/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					};
					if (vErrors === null) vErrors = [err0];
					else vErrors.push(err0);
					errors++;
				} else {
					const _errs3 = errors;
					for (const key0 in data) if (!(key0 === "environment" || key0 === "label" || key0 === "providerId" || key0 === "step" || key0 === "workspaceId")) {
						const err1 = {
							instancePath,
							schemaPath: "#/oneOf/0/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err1];
						else vErrors.push(err1);
						errors++;
						break;
					}
					if (_errs3 === errors) {
						if (data.environment !== void 0) {
							const _errs4 = errors;
							if (typeof data.environment !== "string") {
								const err2 = {
									instancePath: instancePath + "/environment",
									schemaPath: "#/oneOf/0/properties/environment/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								};
								if (vErrors === null) vErrors = [err2];
								else vErrors.push(err2);
								errors++;
							}
							var valid1 = _errs4 === errors;
						} else var valid1 = true;
						if (valid1) {
							if (data.label !== void 0) {
								const _errs6 = errors;
								if (typeof data.label !== "string") {
									const err3 = {
										instancePath: instancePath + "/label",
										schemaPath: "#/oneOf/0/properties/label/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err3];
									else vErrors.push(err3);
									errors++;
								}
								var valid1 = _errs6 === errors;
							} else var valid1 = true;
							if (valid1) {
								if (data.providerId !== void 0) {
									const _errs8 = errors;
									if (typeof data.providerId !== "string") {
										const err4 = {
											instancePath: instancePath + "/providerId",
											schemaPath: "#/oneOf/0/properties/providerId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err4];
										else vErrors.push(err4);
										errors++;
									}
									var valid1 = _errs8 === errors;
								} else var valid1 = true;
								if (valid1) {
									if (data.step !== void 0) {
										let data3 = data.step;
										const _errs10 = errors;
										if (typeof data3 !== "string") {
											const err5 = {
												instancePath: instancePath + "/step",
												schemaPath: "#/oneOf/0/properties/step/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err5];
											else vErrors.push(err5);
											errors++;
										}
										if ("test" !== data3) {
											const err6 = {
												instancePath: instancePath + "/step",
												schemaPath: "#/oneOf/0/properties/step/const",
												keyword: "const",
												params: { allowedValue: "test" },
												message: "must be equal to constant"
											};
											if (vErrors === null) vErrors = [err6];
											else vErrors.push(err6);
											errors++;
										}
										var valid1 = _errs10 === errors;
									} else var valid1 = true;
									if (valid1) {
										if (data.workspaceId !== void 0) {
											const _errs12 = errors;
											if (typeof data.workspaceId !== "string") {
												const err7 = {
													instancePath: instancePath + "/workspaceId",
													schemaPath: "#/oneOf/0/properties/workspaceId/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err7];
												else vErrors.push(err7);
												errors++;
											}
											var valid1 = _errs12 === errors;
										} else var valid1 = true;
									}
								}
							}
						}
					}
				}
			} else {
				const err8 = {
					instancePath,
					schemaPath: "#/oneOf/0/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
			var props0 = true;
		}
		const _errs14 = errors;
		if (errors === _errs14) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing1;
				if (data.step === void 0 && (missing1 = "step") || data.workspaceId === void 0 && (missing1 = "workspaceId") || data.connectionId === void 0 && (missing1 = "connectionId") || data.expectedStateVersion === void 0 && (missing1 = "expectedStateVersion") || data.acknowledgeUnverified === void 0 && (missing1 = "acknowledgeUnverified")) {
					const err9 = {
						instancePath,
						schemaPath: "#/oneOf/1/required",
						keyword: "required",
						params: { missingProperty: missing1 },
						message: "must have required property '" + missing1 + "'"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				} else {
					const _errs16 = errors;
					for (const key1 in data) if (!(key1 === "acknowledgeUnverified" || key1 === "connectionId" || key1 === "expectedStateVersion" || key1 === "step" || key1 === "workspaceId")) {
						const err10 = {
							instancePath,
							schemaPath: "#/oneOf/1/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
						break;
					}
					if (_errs16 === errors) {
						if (data.acknowledgeUnverified !== void 0) {
							const _errs17 = errors;
							if (typeof data.acknowledgeUnverified !== "boolean") {
								const err11 = {
									instancePath: instancePath + "/acknowledgeUnverified",
									schemaPath: "#/oneOf/1/properties/acknowledgeUnverified/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								};
								if (vErrors === null) vErrors = [err11];
								else vErrors.push(err11);
								errors++;
							}
							var valid2 = _errs17 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.connectionId !== void 0) {
								const _errs19 = errors;
								if (typeof data.connectionId !== "string") {
									const err12 = {
										instancePath: instancePath + "/connectionId",
										schemaPath: "#/oneOf/1/properties/connectionId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									if (vErrors === null) vErrors = [err12];
									else vErrors.push(err12);
									errors++;
								}
								var valid2 = _errs19 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.expectedStateVersion !== void 0) {
									const _errs21 = errors;
									if (typeof data.expectedStateVersion !== "string") {
										const err13 = {
											instancePath: instancePath + "/expectedStateVersion",
											schemaPath: "#/oneOf/1/properties/expectedStateVersion/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err13];
										else vErrors.push(err13);
										errors++;
									}
									var valid2 = _errs21 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.step !== void 0) {
										let data8 = data.step;
										const _errs23 = errors;
										if (typeof data8 !== "string") {
											const err14 = {
												instancePath: instancePath + "/step",
												schemaPath: "#/oneOf/1/properties/step/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err14];
											else vErrors.push(err14);
											errors++;
										}
										if ("confirm" !== data8) {
											const err15 = {
												instancePath: instancePath + "/step",
												schemaPath: "#/oneOf/1/properties/step/const",
												keyword: "const",
												params: { allowedValue: "confirm" },
												message: "must be equal to constant"
											};
											if (vErrors === null) vErrors = [err15];
											else vErrors.push(err15);
											errors++;
										}
										var valid2 = _errs23 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.workspaceId !== void 0) {
											const _errs25 = errors;
											if (typeof data.workspaceId !== "string") {
												const err16 = {
													instancePath: instancePath + "/workspaceId",
													schemaPath: "#/oneOf/1/properties/workspaceId/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err16];
												else vErrors.push(err16);
												errors++;
											}
											var valid2 = _errs25 === errors;
										} else var valid2 = true;
									}
								}
							}
						}
					}
				}
			} else {
				const err17 = {
					instancePath,
					schemaPath: "#/oneOf/1/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err17];
				else vErrors.push(err17);
				errors++;
			}
		}
		var _valid0 = _errs14 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err18 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err18];
			else vErrors.push(err18);
			errors++;
			validate97.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate97.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate97.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.ConnectionState = validate98;
	function validate98(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate98.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate98.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "CONNECTING" || data === "REVIEW_REQUIRED" || data === "CONNECTED" || data === "FAILED" || data === "DISCONNECTED")) {
			validate98.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema63.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate98.errors = vErrors;
		return true;
	}
	validate98.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.DomainEvent = validate99;
	var schema41 = {
		"type": "object",
		"properties": {
			"aggregateId": {
				"type": "string",
				"minLength": 1
			},
			"aggregateType": {
				"type": "string",
				"enum": [
					"workspace",
					"account",
					"model-gateway",
					"model",
					"risk"
				]
			},
			"eventId": {
				"type": "string",
				"minLength": 1
			},
			"eventType": {
				"type": "string",
				"enum": [
					"workspace.opened",
					"account.health.changed",
					"model.gateway.changed",
					"model.provider.changed",
					"model.provider_attempt.changed",
					"risk.policy.changed"
				]
			},
			"occurredAt": { "type": "string" },
			"payload": { "$ref": "#/$defs/DomainProjection" },
			"schemaVersion": {
				"type": "integer",
				"format": "uint32",
				"const": 1,
				"minimum": 0
			},
			"sequence": {
				"type": "integer",
				"format": "uint64",
				"maximum": 9007199254740991,
				"minimum": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"eventId",
			"eventType",
			"schemaVersion",
			"occurredAt",
			"aggregateType",
			"aggregateId",
			"sequence",
			"payload"
		]
	};
	var schema43 = {
		"type": "object",
		"properties": {
			"desiredRunning": { "type": "boolean" },
			"discoveredModelCount": {
				"type": "integer",
				"format": "uint32",
				"maximum": 1e3,
				"minimum": 0
			},
			"endpoint": {
				"type": "string",
				"const": "http://127.0.0.1:8317"
			},
			"errorCode": { "type": ["string", "null"] },
			"installed": { "type": "boolean" },
			"lastProbeAt": { "type": ["string", "null"] },
			"modelAvailable": { "type": "boolean" },
			"nextRetryAt": { "type": ["string", "null"] },
			"pinnedVersion": {
				"type": "string",
				"const": "7.2.155"
			},
			"restartAttempts": {
				"type": "integer",
				"format": "uint32",
				"maximum": 3,
				"minimum": 0
			},
			"stateVersion": {
				"type": "string",
				"maxLength": 256,
				"minLength": 1
			},
			"status": { "$ref": "#/$defs/GatewayStatus" },
			"updatedAt": { "type": "string" },
			"workspaceId": {
				"type": "string",
				"maxLength": 128,
				"minLength": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"workspaceId",
			"stateVersion",
			"pinnedVersion",
			"endpoint",
			"status",
			"desiredRunning",
			"installed",
			"modelAvailable",
			"discoveredModelCount",
			"restartAttempts",
			"updatedAt"
		]
	};
	var schema44 = {
		"type": "string",
		"enum": [
			"STOPPED",
			"INSTALLING",
			"STARTING",
			"RUNNING",
			"PORT_CONFLICT",
			"UNAUTHORIZED",
			"BACKOFF",
			"FAILED",
			"STOPPING"
		]
	};
	function validate25(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate25.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.pinnedVersion === void 0 && (missing0 = "pinnedVersion") || data.endpoint === void 0 && (missing0 = "endpoint") || data.status === void 0 && (missing0 = "status") || data.desiredRunning === void 0 && (missing0 = "desiredRunning") || data.installed === void 0 && (missing0 = "installed") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.discoveredModelCount === void 0 && (missing0 = "discoveredModelCount") || data.restartAttempts === void 0 && (missing0 = "restartAttempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate25.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema43.properties, key0)) {
					validate25.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.desiredRunning !== void 0) {
					if (typeof data.desiredRunning !== "boolean") {
						validate25.errors = [{
							instancePath: instancePath + "/desiredRunning",
							schemaPath: "#/properties/desiredRunning/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.discoveredModelCount !== void 0) {
						let data1 = data.discoveredModelCount;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1))) {
							validate25.errors = [{
								instancePath: instancePath + "/discoveredModelCount",
								schemaPath: "#/properties/discoveredModelCount/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							}];
							return false;
						}
						if (typeof data1 == "number") {
							if (data1 > 1e3 || isNaN(data1)) {
								validate25.errors = [{
									instancePath: instancePath + "/discoveredModelCount",
									schemaPath: "#/properties/discoveredModelCount/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 1e3
									},
									message: "must be <= 1000"
								}];
								return false;
							} else if (data1 < 0 || isNaN(data1)) {
								validate25.errors = [{
									instancePath: instancePath + "/discoveredModelCount",
									schemaPath: "#/properties/discoveredModelCount/minimum",
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
						if (data.endpoint !== void 0) {
							let data2 = data.endpoint;
							if (typeof data2 !== "string") {
								validate25.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if ("http://127.0.0.1:8317" !== data2) {
								validate25.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/const",
									keyword: "const",
									params: { allowedValue: "http://127.0.0.1:8317" },
									message: "must be equal to constant"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.errorCode !== void 0) {
								let data3 = data.errorCode;
								if (typeof data3 !== "string" && data3 !== null) {
									validate25.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema43.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.installed !== void 0) {
									if (typeof data.installed !== "boolean") {
										validate25.errors = [{
											instancePath: instancePath + "/installed",
											schemaPath: "#/properties/installed/type",
											keyword: "type",
											params: { type: "boolean" },
											message: "must be boolean"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.lastProbeAt !== void 0) {
										let data5 = data.lastProbeAt;
										if (typeof data5 !== "string" && data5 !== null) {
											validate25.errors = [{
												instancePath: instancePath + "/lastProbeAt",
												schemaPath: "#/properties/lastProbeAt/type",
												keyword: "type",
												params: { type: schema43.properties.lastProbeAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelAvailable !== void 0) {
											if (typeof data.modelAvailable !== "boolean") {
												validate25.errors = [{
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
										if (valid0) {
											if (data.nextRetryAt !== void 0) {
												let data7 = data.nextRetryAt;
												if (typeof data7 !== "string" && data7 !== null) {
													validate25.errors = [{
														instancePath: instancePath + "/nextRetryAt",
														schemaPath: "#/properties/nextRetryAt/type",
														keyword: "type",
														params: { type: schema43.properties.nextRetryAt.type },
														message: "must be string,null"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.pinnedVersion !== void 0) {
													let data8 = data.pinnedVersion;
													if (typeof data8 !== "string") {
														validate25.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if ("7.2.155" !== data8) {
														validate25.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/const",
															keyword: "const",
															params: { allowedValue: "7.2.155" },
															message: "must be equal to constant"
														}];
														return false;
													}
													var valid0 = true;
												} else var valid0 = true;
												if (valid0) {
													if (data.restartAttempts !== void 0) {
														let data9 = data.restartAttempts;
														if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9))) {
															validate25.errors = [{
																instancePath: instancePath + "/restartAttempts",
																schemaPath: "#/properties/restartAttempts/type",
																keyword: "type",
																params: { type: "integer" },
																message: "must be integer"
															}];
															return false;
														}
														if (typeof data9 == "number") {
															if (data9 > 3 || isNaN(data9)) {
																validate25.errors = [{
																	instancePath: instancePath + "/restartAttempts",
																	schemaPath: "#/properties/restartAttempts/maximum",
																	keyword: "maximum",
																	params: {
																		comparison: "<=",
																		limit: 3
																	},
																	message: "must be <= 3"
																}];
																return false;
															} else if (data9 < 0 || isNaN(data9)) {
																validate25.errors = [{
																	instancePath: instancePath + "/restartAttempts",
																	schemaPath: "#/properties/restartAttempts/minimum",
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
														if (data.stateVersion !== void 0) {
															let data10 = data.stateVersion;
															if (typeof data10 === "string") {
																if (func1(data10) > 256) {
																	validate25.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data10) < 1) {
																	validate25.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate25.errors = [{
																	instancePath: instancePath + "/stateVersion",
																	schemaPath: "#/properties/stateVersion/type",
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
																let data11 = data.status;
																if (typeof data11 !== "string") {
																	validate25.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																if (!(data11 === "STOPPED" || data11 === "INSTALLING" || data11 === "STARTING" || data11 === "RUNNING" || data11 === "PORT_CONFLICT" || data11 === "UNAUTHORIZED" || data11 === "BACKOFF" || data11 === "FAILED" || data11 === "STOPPING")) {
																	validate25.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/enum",
																		keyword: "enum",
																		params: { allowedValues: schema44.enum },
																		message: "must be equal to one of the allowed values"
																	}];
																	return false;
																}
																var valid0 = true;
															} else var valid0 = true;
															if (valid0) {
																if (data.updatedAt !== void 0) {
																	if (typeof data.updatedAt !== "string") {
																		validate25.errors = [{
																			instancePath: instancePath + "/updatedAt",
																			schemaPath: "#/properties/updatedAt/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid0 = true;
																} else var valid0 = true;
																if (valid0) {
																	if (data.workspaceId !== void 0) {
																		let data13 = data.workspaceId;
																		if (typeof data13 === "string") {
																			if (func1(data13) > 128) {
																				validate25.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/maxLength",
																					keyword: "maxLength",
																					params: { limit: 128 },
																					message: "must NOT have more than 128 characters"
																				}];
																				return false;
																			} else if (func1(data13) < 1) {
																				validate25.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate25.errors = [{
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
									}
								}
							}
						}
					}
				}
			}
		} else {
			validate25.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate25.errors = vErrors;
		return true;
	}
	validate25.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	var schema45 = {
		"type": "object",
		"properties": {
			"attempts": {
				"type": "array",
				"items": { "$ref": "#/$defs/ModelAttempt" },
				"maxItems": 100
			},
			"automaticFallback": {
				"type": "boolean",
				"default": false
			},
			"chatgpt": { "$ref": "#/$defs/ModelProviderState" },
			"currentRoute": { "anyOf": [{ "$ref": "#/$defs/ModelRoute" }, { "type": "null" }] },
			"deepseek": { "$ref": "#/$defs/ModelProviderState" },
			"defaultRoute": {
				"anyOf": [{ "$ref": "#/$defs/ModelSelection" }, { "type": "null" }],
				"default": null
			},
			"fallbackPolicyVersion": {
				"type": "integer",
				"format": "uint64",
				"default": 1,
				"minimum": 1
			},
			"stateVersion": {
				"type": "string",
				"maxLength": 256,
				"minLength": 1
			},
			"updatedAt": { "type": "string" },
			"workspaceId": {
				"type": "string",
				"maxLength": 128,
				"minLength": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"workspaceId",
			"stateVersion",
			"chatgpt",
			"deepseek",
			"attempts",
			"updatedAt"
		]
	};
	var schema46 = {
		"type": "object",
		"properties": {
			"attemptId": {
				"type": "string",
				"maxLength": 128,
				"minLength": 1
			},
			"endedAt": { "type": "string" },
			"errorCategory": { "type": ["string", "null"] },
			"kind": {
				"$ref": "#/$defs/ModelAttemptKind",
				"default": "SETUP"
			},
			"modelId": { "type": ["string", "null"] },
			"outcome": { "$ref": "#/$defs/ModelAttemptOutcome" },
			"provider": { "$ref": "#/$defs/ModelProvider" },
			"quota": { "anyOf": [{ "$ref": "#/$defs/ModelQuota" }, { "type": "null" }] },
			"startedAt": { "type": "string" },
			"thinkingType": { "anyOf": [{ "$ref": "#/$defs/ThinkingType" }, { "type": "null" }] }
		},
		"additionalProperties": false,
		"required": [
			"attemptId",
			"provider",
			"startedAt",
			"endedAt",
			"outcome"
		]
	};
	var schema47 = {
		"type": "string",
		"enum": ["SETUP", "THREAD"]
	};
	var schema48 = {
		"type": "string",
		"enum": [
			"VERIFIED",
			"CONFIGURED",
			"CANCELLED",
			"FAILED"
		]
	};
	var schema49 = {
		"type": "string",
		"enum": ["CHATGPT", "DEEPSEEK"]
	};
	var schema50 = {
		"type": "object",
		"properties": {
			"remaining": {
				"type": ["integer", "null"],
				"format": "uint64",
				"maximum": 1e9,
				"minimum": 0
			},
			"resetAt": { "type": ["string", "null"] },
			"retryAfterSeconds": {
				"type": ["integer", "null"],
				"format": "uint64",
				"maximum": 86400,
				"minimum": 0
			},
			"window": { "type": ["string", "null"] }
		},
		"additionalProperties": false
	};
	var schema51 = {
		"type": "string",
		"enum": ["disabled", "enabled"]
	};
	function validate28(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate28.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.attemptId === void 0 && (missing0 = "attemptId") || data.provider === void 0 && (missing0 = "provider") || data.startedAt === void 0 && (missing0 = "startedAt") || data.endedAt === void 0 && (missing0 = "endedAt") || data.outcome === void 0 && (missing0 = "outcome")) {
					validate28.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema46.properties, key0)) {
						validate28.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.attemptId !== void 0) {
							let data0 = data.attemptId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate28.errors = [{
											instancePath: instancePath + "/attemptId",
											schemaPath: "#/properties/attemptId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate28.errors = [{
											instancePath: instancePath + "/attemptId",
											schemaPath: "#/properties/attemptId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate28.errors = [{
										instancePath: instancePath + "/attemptId",
										schemaPath: "#/properties/attemptId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.endedAt !== void 0) {
								const _errs4 = errors;
								if (typeof data.endedAt !== "string") {
									validate28.errors = [{
										instancePath: instancePath + "/endedAt",
										schemaPath: "#/properties/endedAt/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.errorCategory !== void 0) {
									let data2 = data.errorCategory;
									const _errs6 = errors;
									if (typeof data2 !== "string" && data2 !== null) {
										validate28.errors = [{
											instancePath: instancePath + "/errorCategory",
											schemaPath: "#/properties/errorCategory/type",
											keyword: "type",
											params: { type: schema46.properties.errorCategory.type },
											message: "must be string,null"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.kind !== void 0) {
										let data3 = data.kind;
										const _errs8 = errors;
										if (typeof data3 !== "string") {
											validate28.errors = [{
												instancePath: instancePath + "/kind",
												schemaPath: "#/$defs/ModelAttemptKind/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "SETUP" || data3 === "THREAD")) {
											validate28.errors = [{
												instancePath: instancePath + "/kind",
												schemaPath: "#/$defs/ModelAttemptKind/enum",
												keyword: "enum",
												params: { allowedValues: schema47.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelId !== void 0) {
											let data4 = data.modelId;
											const _errs11 = errors;
											if (typeof data4 !== "string" && data4 !== null) {
												validate28.errors = [{
													instancePath: instancePath + "/modelId",
													schemaPath: "#/properties/modelId/type",
													keyword: "type",
													params: { type: schema46.properties.modelId.type },
													message: "must be string,null"
												}];
												return false;
											}
											var valid0 = _errs11 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.outcome !== void 0) {
												let data5 = data.outcome;
												const _errs13 = errors;
												if (typeof data5 !== "string") {
													validate28.errors = [{
														instancePath: instancePath + "/outcome",
														schemaPath: "#/$defs/ModelAttemptOutcome/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												if (!(data5 === "VERIFIED" || data5 === "CONFIGURED" || data5 === "CANCELLED" || data5 === "FAILED")) {
													validate28.errors = [{
														instancePath: instancePath + "/outcome",
														schemaPath: "#/$defs/ModelAttemptOutcome/enum",
														keyword: "enum",
														params: { allowedValues: schema48.enum },
														message: "must be equal to one of the allowed values"
													}];
													return false;
												}
												var valid0 = _errs13 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.provider !== void 0) {
													let data6 = data.provider;
													const _errs16 = errors;
													if (typeof data6 !== "string") {
														validate28.errors = [{
															instancePath: instancePath + "/provider",
															schemaPath: "#/$defs/ModelProvider/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if (!(data6 === "CHATGPT" || data6 === "DEEPSEEK")) {
														validate28.errors = [{
															instancePath: instancePath + "/provider",
															schemaPath: "#/$defs/ModelProvider/enum",
															keyword: "enum",
															params: { allowedValues: schema49.enum },
															message: "must be equal to one of the allowed values"
														}];
														return false;
													}
													var valid0 = _errs16 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.quota !== void 0) {
														let data7 = data.quota;
														const _errs19 = errors;
														const _errs20 = errors;
														let valid4 = false;
														const _errs21 = errors;
														if (errors === errors) {
															if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
																const _errs24 = errors;
																for (const key1 in data7) if (!(key1 === "remaining" || key1 === "resetAt" || key1 === "retryAfterSeconds" || key1 === "window")) {
																	const err0 = {
																		instancePath: instancePath + "/quota",
																		schemaPath: "#/$defs/ModelQuota/additionalProperties",
																		keyword: "additionalProperties",
																		params: { additionalProperty: key1 },
																		message: "must NOT have additional properties"
																	};
																	if (vErrors === null) vErrors = [err0];
																	else vErrors.push(err0);
																	errors++;
																	break;
																}
																if (_errs24 === errors) {
																	if (data7.remaining !== void 0) {
																		let data8 = data7.remaining;
																		const _errs25 = errors;
																		if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8)) && data8 !== null) {
																			const err1 = {
																				instancePath: instancePath + "/quota/remaining",
																				schemaPath: "#/$defs/ModelQuota/properties/remaining/type",
																				keyword: "type",
																				params: { type: schema50.properties.remaining.type },
																				message: "must be integer,null"
																			};
																			if (vErrors === null) vErrors = [err1];
																			else vErrors.push(err1);
																			errors++;
																		}
																		if (errors === _errs25) {
																			if (typeof data8 == "number") {
																				if (data8 > 1e9 || isNaN(data8)) {
																					const err2 = {
																						instancePath: instancePath + "/quota/remaining",
																						schemaPath: "#/$defs/ModelQuota/properties/remaining/maximum",
																						keyword: "maximum",
																						params: {
																							comparison: "<=",
																							limit: 1e9
																						},
																						message: "must be <= 1000000000"
																					};
																					if (vErrors === null) vErrors = [err2];
																					else vErrors.push(err2);
																					errors++;
																				} else if (data8 < 0 || isNaN(data8)) {
																					const err3 = {
																						instancePath: instancePath + "/quota/remaining",
																						schemaPath: "#/$defs/ModelQuota/properties/remaining/minimum",
																						keyword: "minimum",
																						params: {
																							comparison: ">=",
																							limit: 0
																						},
																						message: "must be >= 0"
																					};
																					if (vErrors === null) vErrors = [err3];
																					else vErrors.push(err3);
																					errors++;
																				}
																			}
																		}
																		var valid6 = _errs25 === errors;
																	} else var valid6 = true;
																	if (valid6) {
																		if (data7.resetAt !== void 0) {
																			let data9 = data7.resetAt;
																			const _errs27 = errors;
																			if (typeof data9 !== "string" && data9 !== null) {
																				const err4 = {
																					instancePath: instancePath + "/quota/resetAt",
																					schemaPath: "#/$defs/ModelQuota/properties/resetAt/type",
																					keyword: "type",
																					params: { type: schema50.properties.resetAt.type },
																					message: "must be string,null"
																				};
																				if (vErrors === null) vErrors = [err4];
																				else vErrors.push(err4);
																				errors++;
																			}
																			var valid6 = _errs27 === errors;
																		} else var valid6 = true;
																		if (valid6) {
																			if (data7.retryAfterSeconds !== void 0) {
																				let data10 = data7.retryAfterSeconds;
																				const _errs29 = errors;
																				if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10)) && data10 !== null) {
																					const err5 = {
																						instancePath: instancePath + "/quota/retryAfterSeconds",
																						schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/type",
																						keyword: "type",
																						params: { type: schema50.properties.retryAfterSeconds.type },
																						message: "must be integer,null"
																					};
																					if (vErrors === null) vErrors = [err5];
																					else vErrors.push(err5);
																					errors++;
																				}
																				if (errors === _errs29) {
																					if (typeof data10 == "number") {
																						if (data10 > 86400 || isNaN(data10)) {
																							const err6 = {
																								instancePath: instancePath + "/quota/retryAfterSeconds",
																								schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/maximum",
																								keyword: "maximum",
																								params: {
																									comparison: "<=",
																									limit: 86400
																								},
																								message: "must be <= 86400"
																							};
																							if (vErrors === null) vErrors = [err6];
																							else vErrors.push(err6);
																							errors++;
																						} else if (data10 < 0 || isNaN(data10)) {
																							const err7 = {
																								instancePath: instancePath + "/quota/retryAfterSeconds",
																								schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/minimum",
																								keyword: "minimum",
																								params: {
																									comparison: ">=",
																									limit: 0
																								},
																								message: "must be >= 0"
																							};
																							if (vErrors === null) vErrors = [err7];
																							else vErrors.push(err7);
																							errors++;
																						}
																					}
																				}
																				var valid6 = _errs29 === errors;
																			} else var valid6 = true;
																			if (valid6) {
																				if (data7.window !== void 0) {
																					let data11 = data7.window;
																					const _errs31 = errors;
																					if (typeof data11 !== "string" && data11 !== null) {
																						const err8 = {
																							instancePath: instancePath + "/quota/window",
																							schemaPath: "#/$defs/ModelQuota/properties/window/type",
																							keyword: "type",
																							params: { type: schema50.properties.window.type },
																							message: "must be string,null"
																						};
																						if (vErrors === null) vErrors = [err8];
																						else vErrors.push(err8);
																						errors++;
																					}
																					var valid6 = _errs31 === errors;
																				} else var valid6 = true;
																			}
																		}
																	}
																}
															} else {
																const err9 = {
																	instancePath: instancePath + "/quota",
																	schemaPath: "#/$defs/ModelQuota/type",
																	keyword: "type",
																	params: { type: "object" },
																	message: "must be object"
																};
																if (vErrors === null) vErrors = [err9];
																else vErrors.push(err9);
																errors++;
															}
														}
														var _valid0 = _errs21 === errors;
														valid4 = valid4 || _valid0;
														const _errs33 = errors;
														if (data7 !== null) {
															const err10 = {
																instancePath: instancePath + "/quota",
																schemaPath: "#/properties/quota/anyOf/1/type",
																keyword: "type",
																params: { type: "null" },
																message: "must be null"
															};
															if (vErrors === null) vErrors = [err10];
															else vErrors.push(err10);
															errors++;
														}
														var _valid0 = _errs33 === errors;
														valid4 = valid4 || _valid0;
														if (!valid4) {
															const err11 = {
																instancePath: instancePath + "/quota",
																schemaPath: "#/properties/quota/anyOf",
																keyword: "anyOf",
																params: {},
																message: "must match a schema in anyOf"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
															validate28.errors = vErrors;
															return false;
														} else {
															errors = _errs20;
															if (vErrors !== null) {
																if (_errs20) vErrors.length = _errs20;
																else vErrors = null;
															}
														}
														var valid0 = _errs19 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.startedAt !== void 0) {
															const _errs35 = errors;
															if (typeof data.startedAt !== "string") {
																validate28.errors = [{
																	instancePath: instancePath + "/startedAt",
																	schemaPath: "#/properties/startedAt/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid0 = _errs35 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.thinkingType !== void 0) {
																let data13 = data.thinkingType;
																const _errs37 = errors;
																const _errs38 = errors;
																let valid7 = false;
																const _errs39 = errors;
																if (typeof data13 !== "string") {
																	const err12 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/$defs/ThinkingType/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	};
																	if (vErrors === null) vErrors = [err12];
																	else vErrors.push(err12);
																	errors++;
																}
																if (!(data13 === "disabled" || data13 === "enabled")) {
																	const err13 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/$defs/ThinkingType/enum",
																		keyword: "enum",
																		params: { allowedValues: schema51.enum },
																		message: "must be equal to one of the allowed values"
																	};
																	if (vErrors === null) vErrors = [err13];
																	else vErrors.push(err13);
																	errors++;
																}
																var _valid1 = _errs39 === errors;
																valid7 = valid7 || _valid1;
																const _errs42 = errors;
																if (data13 !== null) {
																	const err14 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/properties/thinkingType/anyOf/1/type",
																		keyword: "type",
																		params: { type: "null" },
																		message: "must be null"
																	};
																	if (vErrors === null) vErrors = [err14];
																	else vErrors.push(err14);
																	errors++;
																}
																var _valid1 = _errs42 === errors;
																valid7 = valid7 || _valid1;
																if (!valid7) {
																	const err15 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/properties/thinkingType/anyOf",
																		keyword: "anyOf",
																		params: {},
																		message: "must match a schema in anyOf"
																	};
																	if (vErrors === null) vErrors = [err15];
																	else vErrors.push(err15);
																	errors++;
																	validate28.errors = vErrors;
																	return false;
																} else {
																	errors = _errs38;
																	if (vErrors !== null) {
																		if (_errs38) vErrors.length = _errs38;
																		else vErrors = null;
																	}
																}
																var valid0 = _errs37 === errors;
															} else var valid0 = true;
														}
													}
												}
											}
										}
									}
								}
							}
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
		}
		validate28.errors = vErrors;
		return errors === 0;
	}
	validate28.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	var schema52 = {
		"type": "object",
		"properties": {
			"configured": { "type": "boolean" },
			"errorCode": { "type": ["string", "null"] },
			"lastVerifiedAt": { "type": ["string", "null"] },
			"provider": { "$ref": "#/$defs/ModelProvider" },
			"routes": {
				"type": "array",
				"items": { "$ref": "#/$defs/ModelRoute" },
				"maxItems": 100
			},
			"status": { "$ref": "#/$defs/ModelHealth" }
		},
		"additionalProperties": false,
		"required": [
			"provider",
			"configured",
			"status",
			"routes"
		]
	};
	var schema57 = {
		"type": "string",
		"enum": [
			"NOT_CONFIGURED",
			"UNVERIFIED",
			"VERIFYING",
			"READY",
			"FAILED"
		]
	};
	var schema54 = {
		"type": "object",
		"properties": {
			"modelId": {
				"type": "string",
				"maxLength": 128,
				"minLength": 1
			},
			"provider": { "$ref": "#/$defs/ModelProvider" },
			"thinkingType": { "anyOf": [{ "$ref": "#/$defs/ThinkingType" }, { "type": "null" }] },
			"verifiedAt": { "type": ["string", "null"] }
		},
		"additionalProperties": false,
		"required": ["provider", "modelId"]
	};
	function validate31(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate31.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate31.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "modelId" || key0 === "provider" || key0 === "thinkingType" || key0 === "verifiedAt")) {
						validate31.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.modelId !== void 0) {
							let data0 = data.modelId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate31.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate31.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate31.errors = [{
										instancePath: instancePath + "/modelId",
										schemaPath: "#/properties/modelId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.provider !== void 0) {
								let data1 = data.provider;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate31.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CHATGPT" || data1 === "DEEPSEEK")) {
									validate31.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/enum",
										keyword: "enum",
										params: { allowedValues: schema49.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.thinkingType !== void 0) {
									let data2 = data.thinkingType;
									const _errs7 = errors;
									const _errs8 = errors;
									let valid2 = false;
									const _errs9 = errors;
									if (typeof data2 !== "string") {
										const err0 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err0];
										else vErrors.push(err0);
										errors++;
									}
									if (!(data2 === "disabled" || data2 === "enabled")) {
										const err1 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/enum",
											keyword: "enum",
											params: { allowedValues: schema51.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err1];
										else vErrors.push(err1);
										errors++;
									}
									var _valid0 = _errs9 === errors;
									valid2 = valid2 || _valid0;
									const _errs12 = errors;
									if (data2 !== null) {
										const err2 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf/1/type",
											keyword: "type",
											params: { type: "null" },
											message: "must be null"
										};
										if (vErrors === null) vErrors = [err2];
										else vErrors.push(err2);
										errors++;
									}
									var _valid0 = _errs12 === errors;
									valid2 = valid2 || _valid0;
									if (!valid2) {
										const err3 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										if (vErrors === null) vErrors = [err3];
										else vErrors.push(err3);
										errors++;
										validate31.errors = vErrors;
										return false;
									} else {
										errors = _errs8;
										if (vErrors !== null) {
											if (_errs8) vErrors.length = _errs8;
											else vErrors = null;
										}
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.verifiedAt !== void 0) {
										let data3 = data.verifiedAt;
										const _errs14 = errors;
										if (typeof data3 !== "string" && data3 !== null) {
											validate31.errors = [{
												instancePath: instancePath + "/verifiedAt",
												schemaPath: "#/properties/verifiedAt/type",
												keyword: "type",
												params: { type: schema54.properties.verifiedAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = _errs14 === errors;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			} else {
				validate31.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate31.errors = vErrors;
		return errors === 0;
	}
	validate31.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate30(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate30.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.configured === void 0 && (missing0 = "configured") || data.status === void 0 && (missing0 = "status") || data.routes === void 0 && (missing0 = "routes")) {
					validate30.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "configured" || key0 === "errorCode" || key0 === "lastVerifiedAt" || key0 === "provider" || key0 === "routes" || key0 === "status")) {
						validate30.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.configured !== void 0) {
							const _errs2 = errors;
							if (typeof data.configured !== "boolean") {
								validate30.errors = [{
									instancePath: instancePath + "/configured",
									schemaPath: "#/properties/configured/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.errorCode !== void 0) {
								let data1 = data.errorCode;
								const _errs4 = errors;
								if (typeof data1 !== "string" && data1 !== null) {
									validate30.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema52.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.lastVerifiedAt !== void 0) {
									let data2 = data.lastVerifiedAt;
									const _errs6 = errors;
									if (typeof data2 !== "string" && data2 !== null) {
										validate30.errors = [{
											instancePath: instancePath + "/lastVerifiedAt",
											schemaPath: "#/properties/lastVerifiedAt/type",
											keyword: "type",
											params: { type: schema52.properties.lastVerifiedAt.type },
											message: "must be string,null"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.provider !== void 0) {
										let data3 = data.provider;
										const _errs8 = errors;
										if (typeof data3 !== "string") {
											validate30.errors = [{
												instancePath: instancePath + "/provider",
												schemaPath: "#/$defs/ModelProvider/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "CHATGPT" || data3 === "DEEPSEEK")) {
											validate30.errors = [{
												instancePath: instancePath + "/provider",
												schemaPath: "#/$defs/ModelProvider/enum",
												keyword: "enum",
												params: { allowedValues: schema49.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.routes !== void 0) {
											let data4 = data.routes;
											const _errs11 = errors;
											if (errors === _errs11) {
												if (Array.isArray(data4)) {
													if (data4.length > 100) {
														validate30.errors = [{
															instancePath: instancePath + "/routes",
															schemaPath: "#/properties/routes/maxItems",
															keyword: "maxItems",
															params: { limit: 100 },
															message: "must NOT have more than 100 items"
														}];
														return false;
													} else {
														const len0 = data4.length;
														for (let i0 = 0; i0 < len0; i0++) {
															const _errs13 = errors;
															if (!validate31(data4[i0], {
																instancePath: instancePath + "/routes/" + i0,
																parentData: data4,
																parentDataProperty: i0,
																rootData,
																dynamicAnchors
															})) {
																vErrors = vErrors === null ? validate31.errors : vErrors.concat(validate31.errors);
																errors = vErrors.length;
															}
															if (!(_errs13 === errors)) break;
														}
													}
												} else {
													validate30.errors = [{
														instancePath: instancePath + "/routes",
														schemaPath: "#/properties/routes/type",
														keyword: "type",
														params: { type: "array" },
														message: "must be array"
													}];
													return false;
												}
											}
											var valid0 = _errs11 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.status !== void 0) {
												let data6 = data.status;
												const _errs14 = errors;
												if (typeof data6 !== "string") {
													validate30.errors = [{
														instancePath: instancePath + "/status",
														schemaPath: "#/$defs/ModelHealth/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												if (!(data6 === "NOT_CONFIGURED" || data6 === "UNVERIFIED" || data6 === "VERIFYING" || data6 === "READY" || data6 === "FAILED")) {
													validate30.errors = [{
														instancePath: instancePath + "/status",
														schemaPath: "#/$defs/ModelHealth/enum",
														keyword: "enum",
														params: { allowedValues: schema57.enum },
														message: "must be equal to one of the allowed values"
													}];
													return false;
												}
												var valid0 = _errs14 === errors;
											} else var valid0 = true;
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate30.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate30.errors = vErrors;
		return errors === 0;
	}
	validate30.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate36(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate36.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate36.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "modelId" || key0 === "provider" || key0 === "thinkingType")) {
						validate36.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.modelId !== void 0) {
							let data0 = data.modelId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate36.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate36.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate36.errors = [{
										instancePath: instancePath + "/modelId",
										schemaPath: "#/properties/modelId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.provider !== void 0) {
								let data1 = data.provider;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate36.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CHATGPT" || data1 === "DEEPSEEK")) {
									validate36.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/enum",
										keyword: "enum",
										params: { allowedValues: schema49.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.thinkingType !== void 0) {
									let data2 = data.thinkingType;
									const _errs7 = errors;
									const _errs8 = errors;
									let valid2 = false;
									const _errs9 = errors;
									if (typeof data2 !== "string") {
										const err0 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err0];
										else vErrors.push(err0);
										errors++;
									}
									if (!(data2 === "disabled" || data2 === "enabled")) {
										const err1 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/enum",
											keyword: "enum",
											params: { allowedValues: schema51.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err1];
										else vErrors.push(err1);
										errors++;
									}
									var _valid0 = _errs9 === errors;
									valid2 = valid2 || _valid0;
									const _errs12 = errors;
									if (data2 !== null) {
										const err2 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf/1/type",
											keyword: "type",
											params: { type: "null" },
											message: "must be null"
										};
										if (vErrors === null) vErrors = [err2];
										else vErrors.push(err2);
										errors++;
									}
									var _valid0 = _errs12 === errors;
									valid2 = valid2 || _valid0;
									if (!valid2) {
										const err3 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										if (vErrors === null) vErrors = [err3];
										else vErrors.push(err3);
										errors++;
										validate36.errors = vErrors;
										return false;
									} else {
										errors = _errs8;
										if (vErrors !== null) {
											if (_errs8) vErrors.length = _errs8;
											else vErrors = null;
										}
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
							}
						}
					}
				}
			} else {
				validate36.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate36.errors = vErrors;
		return errors === 0;
	}
	validate36.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate27(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate27.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.chatgpt === void 0 && (missing0 = "chatgpt") || data.deepseek === void 0 && (missing0 = "deepseek") || data.attempts === void 0 && (missing0 = "attempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
					validate27.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema45.properties, key0)) {
						validate27.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.attempts !== void 0) {
							let data0 = data.attempts;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (Array.isArray(data0)) {
									if (data0.length > 100) {
										validate27.errors = [{
											instancePath: instancePath + "/attempts",
											schemaPath: "#/properties/attempts/maxItems",
											keyword: "maxItems",
											params: { limit: 100 },
											message: "must NOT have more than 100 items"
										}];
										return false;
									} else {
										const len0 = data0.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs4 = errors;
											if (!validate28(data0[i0], {
												instancePath: instancePath + "/attempts/" + i0,
												parentData: data0,
												parentDataProperty: i0,
												rootData,
												dynamicAnchors
											})) {
												vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
												errors = vErrors.length;
											}
											if (!(_errs4 === errors)) break;
										}
									}
								} else {
									validate27.errors = [{
										instancePath: instancePath + "/attempts",
										schemaPath: "#/properties/attempts/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.automaticFallback !== void 0) {
								const _errs5 = errors;
								if (typeof data.automaticFallback !== "boolean") {
									validate27.errors = [{
										instancePath: instancePath + "/automaticFallback",
										schemaPath: "#/properties/automaticFallback/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								var valid0 = _errs5 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.chatgpt !== void 0) {
									const _errs7 = errors;
									if (!validate30(data.chatgpt, {
										instancePath: instancePath + "/chatgpt",
										parentData: data,
										parentDataProperty: "chatgpt",
										rootData,
										dynamicAnchors
									})) {
										vErrors = vErrors === null ? validate30.errors : vErrors.concat(validate30.errors);
										errors = vErrors.length;
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.currentRoute !== void 0) {
										let data4 = data.currentRoute;
										const _errs8 = errors;
										const _errs9 = errors;
										let valid2 = false;
										const _errs10 = errors;
										if (!validate31(data4, {
											instancePath: instancePath + "/currentRoute",
											parentData: data,
											parentDataProperty: "currentRoute",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate31.errors : vErrors.concat(validate31.errors);
											errors = vErrors.length;
										}
										var _valid0 = _errs10 === errors;
										valid2 = valid2 || _valid0;
										const _errs11 = errors;
										if (data4 !== null) {
											const err0 = {
												instancePath: instancePath + "/currentRoute",
												schemaPath: "#/properties/currentRoute/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err1 = {
												instancePath: instancePath + "/currentRoute",
												schemaPath: "#/properties/currentRoute/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
											validate27.errors = vErrors;
											return false;
										} else {
											errors = _errs9;
											if (vErrors !== null) {
												if (_errs9) vErrors.length = _errs9;
												else vErrors = null;
											}
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.deepseek !== void 0) {
											const _errs13 = errors;
											if (!validate30(data.deepseek, {
												instancePath: instancePath + "/deepseek",
												parentData: data,
												parentDataProperty: "deepseek",
												rootData,
												dynamicAnchors
											})) {
												vErrors = vErrors === null ? validate30.errors : vErrors.concat(validate30.errors);
												errors = vErrors.length;
											}
											var valid0 = _errs13 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.defaultRoute !== void 0) {
												let data6 = data.defaultRoute;
												const _errs14 = errors;
												const _errs15 = errors;
												let valid3 = false;
												const _errs16 = errors;
												if (!validate36(data6, {
													instancePath: instancePath + "/defaultRoute",
													parentData: data,
													parentDataProperty: "defaultRoute",
													rootData,
													dynamicAnchors
												})) {
													vErrors = vErrors === null ? validate36.errors : vErrors.concat(validate36.errors);
													errors = vErrors.length;
												}
												var _valid1 = _errs16 === errors;
												valid3 = valid3 || _valid1;
												const _errs17 = errors;
												if (data6 !== null) {
													const err2 = {
														instancePath: instancePath + "/defaultRoute",
														schemaPath: "#/properties/defaultRoute/anyOf/1/type",
														keyword: "type",
														params: { type: "null" },
														message: "must be null"
													};
													if (vErrors === null) vErrors = [err2];
													else vErrors.push(err2);
													errors++;
												}
												var _valid1 = _errs17 === errors;
												valid3 = valid3 || _valid1;
												if (!valid3) {
													const err3 = {
														instancePath: instancePath + "/defaultRoute",
														schemaPath: "#/properties/defaultRoute/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													if (vErrors === null) vErrors = [err3];
													else vErrors.push(err3);
													errors++;
													validate27.errors = vErrors;
													return false;
												} else {
													errors = _errs15;
													if (vErrors !== null) {
														if (_errs15) vErrors.length = _errs15;
														else vErrors = null;
													}
												}
												var valid0 = _errs14 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.fallbackPolicyVersion !== void 0) {
													let data7 = data.fallbackPolicyVersion;
													const _errs19 = errors;
													if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
														validate27.errors = [{
															instancePath: instancePath + "/fallbackPolicyVersion",
															schemaPath: "#/properties/fallbackPolicyVersion/type",
															keyword: "type",
															params: { type: "integer" },
															message: "must be integer"
														}];
														return false;
													}
													if (errors === _errs19) {
														if (typeof data7 == "number") {
															if (data7 < 1 || isNaN(data7)) {
																validate27.errors = [{
																	instancePath: instancePath + "/fallbackPolicyVersion",
																	schemaPath: "#/properties/fallbackPolicyVersion/minimum",
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
													}
													var valid0 = _errs19 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.stateVersion !== void 0) {
														let data8 = data.stateVersion;
														const _errs21 = errors;
														if (errors === _errs21) {
															if (typeof data8 === "string") {
																if (func1(data8) > 256) {
																	validate27.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data8) < 1) {
																	validate27.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate27.errors = [{
																	instancePath: instancePath + "/stateVersion",
																	schemaPath: "#/properties/stateVersion/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
														}
														var valid0 = _errs21 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.updatedAt !== void 0) {
															const _errs23 = errors;
															if (typeof data.updatedAt !== "string") {
																validate27.errors = [{
																	instancePath: instancePath + "/updatedAt",
																	schemaPath: "#/properties/updatedAt/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid0 = _errs23 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.workspaceId !== void 0) {
																let data10 = data.workspaceId;
																const _errs25 = errors;
																if (errors === _errs25) {
																	if (typeof data10 === "string") {
																		if (func1(data10) > 128) {
																			validate27.errors = [{
																				instancePath: instancePath + "/workspaceId",
																				schemaPath: "#/properties/workspaceId/maxLength",
																				keyword: "maxLength",
																				params: { limit: 128 },
																				message: "must NOT have more than 128 characters"
																			}];
																			return false;
																		} else if (func1(data10) < 1) {
																			validate27.errors = [{
																				instancePath: instancePath + "/workspaceId",
																				schemaPath: "#/properties/workspaceId/minLength",
																				keyword: "minLength",
																				params: { limit: 1 },
																				message: "must NOT have fewer than 1 characters"
																			}];
																			return false;
																		}
																	} else {
																		validate27.errors = [{
																			instancePath: instancePath + "/workspaceId",
																			schemaPath: "#/properties/workspaceId/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																}
																var valid0 = _errs25 === errors;
															} else var valid0 = true;
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate27.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate27.errors = vErrors;
		return errors === 0;
	}
	validate27.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	var schema70 = {
		"type": "object",
		"properties": {
			"configured": { "type": "boolean" },
			"hardRules": {
				"type": "array",
				"items": { "$ref": "#/$defs/HardSafetyRule" }
			},
			"onboardingCompleted": { "type": "boolean" },
			"onboardingStep": {
				"type": "integer",
				"format": "uint8",
				"maximum": 5,
				"minimum": 1
			},
			"policy": { "$ref": "#/$defs/RiskPolicy" },
			"policyVersion": {
				"type": "integer",
				"format": "uint64",
				"minimum": 1
			},
			"stateVersion": {
				"type": "string",
				"maxLength": 256,
				"minLength": 1
			},
			"updatedAt": { "type": "string" },
			"workspaceId": {
				"type": "string",
				"maxLength": 128,
				"minLength": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"workspaceId",
			"stateVersion",
			"policyVersion",
			"configured",
			"onboardingStep",
			"onboardingCompleted",
			"policy",
			"hardRules",
			"updatedAt"
		]
	};
	var schema72 = {
		"type": "object",
		"properties": {
			"liveInactivityTimeoutMinutes": {
				"type": "integer",
				"format": "uint64",
				"default": 20,
				"maximum": 1440,
				"minimum": 1
			},
			"marketOrdersEnabled": {
				"type": "boolean",
				"default": false
			},
			"maxDailyRealizedLoss": {
				"type": ["string", "null"],
				"default": null,
				"maxLength": 32
			},
			"maxDailyTradedNotional": {
				"type": ["string", "null"],
				"default": null,
				"maxLength": 32
			},
			"maxOrderNotional": {
				"type": ["string", "null"],
				"default": null,
				"maxLength": 32
			},
			"maxSingleInstrumentExposurePercent": {
				"type": ["string", "null"],
				"default": null,
				"maxLength": 32
			},
			"staleQuoteThresholdSeconds": {
				"type": "integer",
				"format": "uint64",
				"default": 3,
				"maximum": 86400,
				"minimum": 1
			}
		},
		"additionalProperties": false
	};
	function validate43(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate43.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.policyVersion === void 0 && (missing0 = "policyVersion") || data.configured === void 0 && (missing0 = "configured") || data.onboardingStep === void 0 && (missing0 = "onboardingStep") || data.onboardingCompleted === void 0 && (missing0 = "onboardingCompleted") || data.policy === void 0 && (missing0 = "policy") || data.hardRules === void 0 && (missing0 = "hardRules") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate43.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema70.properties, key0)) {
					validate43.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.configured !== void 0) {
					if (typeof data.configured !== "boolean") {
						validate43.errors = [{
							instancePath: instancePath + "/configured",
							schemaPath: "#/properties/configured/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.hardRules !== void 0) {
						let data1 = data.hardRules;
						if (Array.isArray(data1)) {
							const len0 = data1.length;
							for (let i0 = 0; i0 < len0; i0++) {
								let data2 = data1[i0];
								if (data2 && typeof data2 == "object" && !Array.isArray(data2)) {
									let missing1;
									if (data2.id === void 0 && (missing1 = "id") || data2.description === void 0 && (missing1 = "description")) {
										validate43.errors = [{
											instancePath: instancePath + "/hardRules/" + i0,
											schemaPath: "#/$defs/HardSafetyRule/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "description" || key1 === "id")) {
											validate43.errors = [{
												instancePath: instancePath + "/hardRules/" + i0,
												schemaPath: "#/$defs/HardSafetyRule/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data2.description !== void 0) {
											let data3 = data2.description;
											if (typeof data3 === "string") {
												if (func1(data3) > 256) {
													validate43.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/description",
														schemaPath: "#/$defs/HardSafetyRule/properties/description/maxLength",
														keyword: "maxLength",
														params: { limit: 256 },
														message: "must NOT have more than 256 characters"
													}];
													return false;
												} else if (func1(data3) < 1) {
													validate43.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/description",
														schemaPath: "#/$defs/HardSafetyRule/properties/description/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate43.errors = [{
													instancePath: instancePath + "/hardRules/" + i0 + "/description",
													schemaPath: "#/$defs/HardSafetyRule/properties/description/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data2.id !== void 0) {
												let data4 = data2.id;
												if (typeof data4 === "string") {
													if (func1(data4) > 64) {
														validate43.errors = [{
															instancePath: instancePath + "/hardRules/" + i0 + "/id",
															schemaPath: "#/$defs/HardSafetyRule/properties/id/maxLength",
															keyword: "maxLength",
															params: { limit: 64 },
															message: "must NOT have more than 64 characters"
														}];
														return false;
													} else if (func1(data4) < 1) {
														validate43.errors = [{
															instancePath: instancePath + "/hardRules/" + i0 + "/id",
															schemaPath: "#/$defs/HardSafetyRule/properties/id/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate43.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/id",
														schemaPath: "#/$defs/HardSafetyRule/properties/id/type",
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
									validate43.errors = [{
										instancePath: instancePath + "/hardRules/" + i0,
										schemaPath: "#/$defs/HardSafetyRule/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
							}
						} else {
							validate43.errors = [{
								instancePath: instancePath + "/hardRules",
								schemaPath: "#/properties/hardRules/type",
								keyword: "type",
								params: { type: "array" },
								message: "must be array"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.onboardingCompleted !== void 0) {
							if (typeof data.onboardingCompleted !== "boolean") {
								validate43.errors = [{
									instancePath: instancePath + "/onboardingCompleted",
									schemaPath: "#/properties/onboardingCompleted/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.onboardingStep !== void 0) {
								let data6 = data.onboardingStep;
								if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
									validate43.errors = [{
										instancePath: instancePath + "/onboardingStep",
										schemaPath: "#/properties/onboardingStep/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									}];
									return false;
								}
								if (typeof data6 == "number") {
									if (data6 > 5 || isNaN(data6)) {
										validate43.errors = [{
											instancePath: instancePath + "/onboardingStep",
											schemaPath: "#/properties/onboardingStep/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 5
											},
											message: "must be <= 5"
										}];
										return false;
									} else if (data6 < 1 || isNaN(data6)) {
										validate43.errors = [{
											instancePath: instancePath + "/onboardingStep",
											schemaPath: "#/properties/onboardingStep/minimum",
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
							if (valid0) {
								if (data.policy !== void 0) {
									let data7 = data.policy;
									if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
										for (const key2 in data7) if (!(key2 === "liveInactivityTimeoutMinutes" || key2 === "marketOrdersEnabled" || key2 === "maxDailyRealizedLoss" || key2 === "maxDailyTradedNotional" || key2 === "maxOrderNotional" || key2 === "maxSingleInstrumentExposurePercent" || key2 === "staleQuoteThresholdSeconds")) {
											validate43.errors = [{
												instancePath: instancePath + "/policy",
												schemaPath: "#/$defs/RiskPolicy/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key2 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data7.liveInactivityTimeoutMinutes !== void 0) {
											let data8 = data7.liveInactivityTimeoutMinutes;
											if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8))) {
												validate43.errors = [{
													instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
													schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												}];
												return false;
											}
											if (typeof data8 == "number") {
												if (data8 > 1440 || isNaN(data8)) {
													validate43.errors = [{
														instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
														schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 1440
														},
														message: "must be <= 1440"
													}];
													return false;
												} else if (data8 < 1 || isNaN(data8)) {
													validate43.errors = [{
														instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
														schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/minimum",
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
											var valid5 = true;
										} else var valid5 = true;
										if (valid5) {
											if (data7.marketOrdersEnabled !== void 0) {
												if (typeof data7.marketOrdersEnabled !== "boolean") {
													validate43.errors = [{
														instancePath: instancePath + "/policy/marketOrdersEnabled",
														schemaPath: "#/$defs/RiskPolicy/properties/marketOrdersEnabled/type",
														keyword: "type",
														params: { type: "boolean" },
														message: "must be boolean"
													}];
													return false;
												}
												var valid5 = true;
											} else var valid5 = true;
											if (valid5) {
												if (data7.maxDailyRealizedLoss !== void 0) {
													let data10 = data7.maxDailyRealizedLoss;
													if (typeof data10 !== "string" && data10 !== null) {
														validate43.errors = [{
															instancePath: instancePath + "/policy/maxDailyRealizedLoss",
															schemaPath: "#/$defs/RiskPolicy/properties/maxDailyRealizedLoss/type",
															keyword: "type",
															params: { type: schema72.properties.maxDailyRealizedLoss.type },
															message: "must be string,null"
														}];
														return false;
													}
													if (typeof data10 === "string") {
														if (func1(data10) > 32) {
															validate43.errors = [{
																instancePath: instancePath + "/policy/maxDailyRealizedLoss",
																schemaPath: "#/$defs/RiskPolicy/properties/maxDailyRealizedLoss/maxLength",
																keyword: "maxLength",
																params: { limit: 32 },
																message: "must NOT have more than 32 characters"
															}];
															return false;
														}
													}
													var valid5 = true;
												} else var valid5 = true;
												if (valid5) {
													if (data7.maxDailyTradedNotional !== void 0) {
														let data11 = data7.maxDailyTradedNotional;
														if (typeof data11 !== "string" && data11 !== null) {
															validate43.errors = [{
																instancePath: instancePath + "/policy/maxDailyTradedNotional",
																schemaPath: "#/$defs/RiskPolicy/properties/maxDailyTradedNotional/type",
																keyword: "type",
																params: { type: schema72.properties.maxDailyTradedNotional.type },
																message: "must be string,null"
															}];
															return false;
														}
														if (typeof data11 === "string") {
															if (func1(data11) > 32) {
																validate43.errors = [{
																	instancePath: instancePath + "/policy/maxDailyTradedNotional",
																	schemaPath: "#/$defs/RiskPolicy/properties/maxDailyTradedNotional/maxLength",
																	keyword: "maxLength",
																	params: { limit: 32 },
																	message: "must NOT have more than 32 characters"
																}];
																return false;
															}
														}
														var valid5 = true;
													} else var valid5 = true;
													if (valid5) {
														if (data7.maxOrderNotional !== void 0) {
															let data12 = data7.maxOrderNotional;
															if (typeof data12 !== "string" && data12 !== null) {
																validate43.errors = [{
																	instancePath: instancePath + "/policy/maxOrderNotional",
																	schemaPath: "#/$defs/RiskPolicy/properties/maxOrderNotional/type",
																	keyword: "type",
																	params: { type: schema72.properties.maxOrderNotional.type },
																	message: "must be string,null"
																}];
																return false;
															}
															if (typeof data12 === "string") {
																if (func1(data12) > 32) {
																	validate43.errors = [{
																		instancePath: instancePath + "/policy/maxOrderNotional",
																		schemaPath: "#/$defs/RiskPolicy/properties/maxOrderNotional/maxLength",
																		keyword: "maxLength",
																		params: { limit: 32 },
																		message: "must NOT have more than 32 characters"
																	}];
																	return false;
																}
															}
															var valid5 = true;
														} else var valid5 = true;
														if (valid5) {
															if (data7.maxSingleInstrumentExposurePercent !== void 0) {
																let data13 = data7.maxSingleInstrumentExposurePercent;
																if (typeof data13 !== "string" && data13 !== null) {
																	validate43.errors = [{
																		instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																		schemaPath: "#/$defs/RiskPolicy/properties/maxSingleInstrumentExposurePercent/type",
																		keyword: "type",
																		params: { type: schema72.properties.maxSingleInstrumentExposurePercent.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																if (typeof data13 === "string") {
																	if (func1(data13) > 32) {
																		validate43.errors = [{
																			instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																			schemaPath: "#/$defs/RiskPolicy/properties/maxSingleInstrumentExposurePercent/maxLength",
																			keyword: "maxLength",
																			params: { limit: 32 },
																			message: "must NOT have more than 32 characters"
																		}];
																		return false;
																	}
																}
																var valid5 = true;
															} else var valid5 = true;
															if (valid5) {
																if (data7.staleQuoteThresholdSeconds !== void 0) {
																	let data14 = data7.staleQuoteThresholdSeconds;
																	if (!(typeof data14 == "number" && !(data14 % 1) && !isNaN(data14))) {
																		validate43.errors = [{
																			instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																			schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/type",
																			keyword: "type",
																			params: { type: "integer" },
																			message: "must be integer"
																		}];
																		return false;
																	}
																	if (typeof data14 == "number") {
																		if (data14 > 86400 || isNaN(data14)) {
																			validate43.errors = [{
																				instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																				schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/maximum",
																				keyword: "maximum",
																				params: {
																					comparison: "<=",
																					limit: 86400
																				},
																				message: "must be <= 86400"
																			}];
																			return false;
																		} else if (data14 < 1 || isNaN(data14)) {
																			validate43.errors = [{
																				instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																				schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/minimum",
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
																	var valid5 = true;
																} else var valid5 = true;
															}
														}
													}
												}
											}
										}
									} else {
										validate43.errors = [{
											instancePath: instancePath + "/policy",
											schemaPath: "#/$defs/RiskPolicy/type",
											keyword: "type",
											params: { type: "object" },
											message: "must be object"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.policyVersion !== void 0) {
										let data15 = data.policyVersion;
										if (!(typeof data15 == "number" && !(data15 % 1) && !isNaN(data15))) {
											validate43.errors = [{
												instancePath: instancePath + "/policyVersion",
												schemaPath: "#/properties/policyVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data15 == "number") {
											if (data15 < 1 || isNaN(data15)) {
												validate43.errors = [{
													instancePath: instancePath + "/policyVersion",
													schemaPath: "#/properties/policyVersion/minimum",
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
									if (valid0) {
										if (data.stateVersion !== void 0) {
											let data16 = data.stateVersion;
											if (typeof data16 === "string") {
												if (func1(data16) > 256) {
													validate43.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/maxLength",
														keyword: "maxLength",
														params: { limit: 256 },
														message: "must NOT have more than 256 characters"
													}];
													return false;
												} else if (func1(data16) < 1) {
													validate43.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate43.errors = [{
													instancePath: instancePath + "/stateVersion",
													schemaPath: "#/properties/stateVersion/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.updatedAt !== void 0) {
												if (typeof data.updatedAt !== "string") {
													validate43.errors = [{
														instancePath: instancePath + "/updatedAt",
														schemaPath: "#/properties/updatedAt/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.workspaceId !== void 0) {
													let data18 = data.workspaceId;
													if (typeof data18 === "string") {
														if (func1(data18) > 128) {
															validate43.errors = [{
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/properties/workspaceId/maxLength",
																keyword: "maxLength",
																params: { limit: 128 },
																message: "must NOT have more than 128 characters"
															}];
															return false;
														} else if (func1(data18) < 1) {
															validate43.errors = [{
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/properties/workspaceId/minLength",
																keyword: "minLength",
																params: { limit: 1 },
																message: "must NOT have fewer than 1 characters"
															}];
															return false;
														}
													} else {
														validate43.errors = [{
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
	function validate24(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate24.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate25(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
		if (!validate27(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate27.errors : vErrors.concat(validate27.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs3 = errors;
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
					const _errs6 = errors;
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
					if (_errs6 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs7 = errors;
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
							var valid2 = _errs7 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs9 = errors;
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
								var valid2 = _errs9 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs11 = errors;
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
									var valid2 = _errs11 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs13 = errors;
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
										var valid2 = _errs13 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs15 = errors;
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
											var valid2 = _errs15 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs17 = errors;
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
												if (errors === _errs17) {
													if (typeof data5 == "number") {
														if (data5 > 5 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 5
																},
																message: "must be <= 5"
															};
															if (vErrors === null) vErrors = [err8];
															else vErrors.push(err8);
															errors++;
														} else if (data5 < 1 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 1
																},
																message: "must be >= 1"
															};
															if (vErrors === null) vErrors = [err9];
															else vErrors.push(err9);
															errors++;
														}
													}
												}
												var valid2 = _errs17 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													let data6 = data.workspaceId;
													const _errs19 = errors;
													if (errors === _errs19) {
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																const err10 = {
																	instancePath: instancePath + "/workspaceId",
																	schemaPath: "#/$defs/Workspace/properties/workspaceId/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																};
																if (vErrors === null) vErrors = [err10];
																else vErrors.push(err10);
																errors++;
															}
														} else {
															const err11 = {
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
														}
													}
													var valid2 = _errs19 === errors;
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
				const err12 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
		}
		var _valid0 = _errs3 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs21 = errors;
		if (!validate39(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs22 = errors;
		if (!validate43(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs22 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err13 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err13];
			else vErrors.push(err13);
			errors++;
			validate24.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate24.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate24.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate99(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate99.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.eventId === void 0 && (missing0 = "eventId") || data.eventType === void 0 && (missing0 = "eventType") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.occurredAt === void 0 && (missing0 = "occurredAt") || data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.sequence === void 0 && (missing0 = "sequence") || data.payload === void 0 && (missing0 = "payload")) {
					validate99.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "eventId" || key0 === "eventType" || key0 === "occurredAt" || key0 === "payload" || key0 === "schemaVersion" || key0 === "sequence")) {
						validate99.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.aggregateId !== void 0) {
							let data0 = data.aggregateId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate99.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate99.errors = [{
										instancePath: instancePath + "/aggregateId",
										schemaPath: "#/properties/aggregateId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.aggregateType !== void 0) {
								let data1 = data.aggregateType;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate99.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway" || data1 === "model" || data1 === "risk")) {
									validate99.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema41.properties.aggregateType.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.eventId !== void 0) {
									let data2 = data.eventId;
									const _errs6 = errors;
									if (errors === _errs6) {
										if (typeof data2 === "string") {
											if (func1(data2) < 1) {
												validate99.errors = [{
													instancePath: instancePath + "/eventId",
													schemaPath: "#/properties/eventId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate99.errors = [{
												instancePath: instancePath + "/eventId",
												schemaPath: "#/properties/eventId/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.eventType !== void 0) {
										let data3 = data.eventType;
										const _errs8 = errors;
										if (typeof data3 !== "string") {
											validate99.errors = [{
												instancePath: instancePath + "/eventType",
												schemaPath: "#/properties/eventType/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "workspace.opened" || data3 === "account.health.changed" || data3 === "model.gateway.changed" || data3 === "model.provider.changed" || data3 === "model.provider_attempt.changed" || data3 === "risk.policy.changed")) {
											validate99.errors = [{
												instancePath: instancePath + "/eventType",
												schemaPath: "#/properties/eventType/enum",
												keyword: "enum",
												params: { allowedValues: schema41.properties.eventType.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.occurredAt !== void 0) {
											const _errs10 = errors;
											if (typeof data.occurredAt !== "string") {
												validate99.errors = [{
													instancePath: instancePath + "/occurredAt",
													schemaPath: "#/properties/occurredAt/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = _errs10 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.payload !== void 0) {
												const _errs12 = errors;
												if (!validate24(data.payload, {
													instancePath: instancePath + "/payload",
													parentData: data,
													parentDataProperty: "payload",
													rootData,
													dynamicAnchors
												})) {
													vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
													errors = vErrors.length;
												}
												var valid0 = _errs12 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.schemaVersion !== void 0) {
													let data6 = data.schemaVersion;
													const _errs13 = errors;
													if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
														validate99.errors = [{
															instancePath: instancePath + "/schemaVersion",
															schemaPath: "#/properties/schemaVersion/type",
															keyword: "type",
															params: { type: "integer" },
															message: "must be integer"
														}];
														return false;
													}
													if (1 !== data6) {
														validate99.errors = [{
															instancePath: instancePath + "/schemaVersion",
															schemaPath: "#/properties/schemaVersion/const",
															keyword: "const",
															params: { allowedValue: 1 },
															message: "must be equal to constant"
														}];
														return false;
													}
													if (errors === _errs13) {
														if (typeof data6 == "number") {
															if (data6 < 0 || isNaN(data6)) {
																validate99.errors = [{
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
													var valid0 = _errs13 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.sequence !== void 0) {
														let data7 = data.sequence;
														const _errs15 = errors;
														if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
															validate99.errors = [{
																instancePath: instancePath + "/sequence",
																schemaPath: "#/properties/sequence/type",
																keyword: "type",
																params: { type: "integer" },
																message: "must be integer"
															}];
															return false;
														}
														if (errors === _errs15) {
															if (typeof data7 == "number") {
																if (data7 > 9007199254740991 || isNaN(data7)) {
																	validate99.errors = [{
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
																} else if (data7 < 1 || isNaN(data7)) {
																	validate99.errors = [{
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
														}
														var valid0 = _errs15 === errors;
													} else var valid0 = true;
												}
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate99.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate99.errors = vErrors;
		return errors === 0;
	}
	validate99.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.DomainProjection = validate101;
	function validate101(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate101.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate25(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
		if (!validate27(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate27.errors : vErrors.concat(validate27.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs3 = errors;
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
					const _errs6 = errors;
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
					if (_errs6 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs7 = errors;
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
							var valid2 = _errs7 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs9 = errors;
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
								var valid2 = _errs9 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs11 = errors;
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
									var valid2 = _errs11 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs13 = errors;
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
										var valid2 = _errs13 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs15 = errors;
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
											var valid2 = _errs15 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs17 = errors;
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
												if (errors === _errs17) {
													if (typeof data5 == "number") {
														if (data5 > 5 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 5
																},
																message: "must be <= 5"
															};
															if (vErrors === null) vErrors = [err8];
															else vErrors.push(err8);
															errors++;
														} else if (data5 < 1 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 1
																},
																message: "must be >= 1"
															};
															if (vErrors === null) vErrors = [err9];
															else vErrors.push(err9);
															errors++;
														}
													}
												}
												var valid2 = _errs17 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													let data6 = data.workspaceId;
													const _errs19 = errors;
													if (errors === _errs19) {
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																const err10 = {
																	instancePath: instancePath + "/workspaceId",
																	schemaPath: "#/$defs/Workspace/properties/workspaceId/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																};
																if (vErrors === null) vErrors = [err10];
																else vErrors.push(err10);
																errors++;
															}
														} else {
															const err11 = {
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
														}
													}
													var valid2 = _errs19 === errors;
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
				const err12 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
		}
		var _valid0 = _errs3 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs21 = errors;
		if (!validate39(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs22 = errors;
		if (!validate43(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs22 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err13 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err13];
			else vErrors.push(err13);
			errors++;
			validate101.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate101.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate101.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.EmptyPayload = validate106;
	function validate106(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate106.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) for (const key0 in data) {
			validate106.errors = [{
				instancePath,
				schemaPath: "#/additionalProperties",
				keyword: "additionalProperties",
				params: { additionalProperty: key0 },
				message: "must NOT have additional properties"
			}];
			return false;
		}
		else {
			validate106.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate106.errors = vErrors;
		return true;
	}
	validate106.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.FailureEnvelope = validate107;
	function validate72(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate72.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate72.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate72.errors = [{
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
						validate72.errors = [{
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
							validate72.errors = [{
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
								validate72.errors = [{
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
									validate72.errors = [{
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
													validate72.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate72.errors = [{
															instancePath: instancePath + "/remediationActions/" + i0,
															schemaPath: "#/$defs/Remediation/additionalProperties",
															keyword: "additionalProperties",
															params: { additionalProperty: key1 },
															message: "must NOT have additional properties"
														}];
														return false;
													}
													if (data5.id !== void 0) {
														let data6 = data5.id;
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																validate72.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																	schemaPath: "#/$defs/Remediation/properties/id/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																}];
																return false;
															}
														} else {
															validate72.errors = [{
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
																validate72.errors = [{
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
												validate72.errors = [{
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
										validate72.errors = [{
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
											validate72.errors = [{
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
			validate72.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate72.errors = vErrors;
		return true;
	}
	validate72.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate107(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate107.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate107.errors = [{
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
						validate107.errors = [{
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
							if (!validate72(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate72.errors : vErrors.concat(validate72.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate107.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate107.errors = [{
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
									let data2 = data.requestId;
									const _errs5 = errors;
									if (errors === _errs5) {
										if (typeof data2 === "string") {
											if (func1(data2) > 128) {
												validate107.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate107.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate107.errors = [{
												instancePath: instancePath + "/requestId",
												schemaPath: "#/properties/requestId/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate107.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate107.errors = [{
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
													validate107.errors = [{
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
				validate107.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate107.errors = vErrors;
		return errors === 0;
	}
	validate107.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayAction = validate109;
	var schema74 = {
		"type": "string",
		"enum": [
			"LAUNCH",
			"PROBE",
			"RESTART",
			"STOP"
		]
	};
	function validate109(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate109.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate109.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "LAUNCH" || data === "PROBE" || data === "RESTART" || data === "STOP")) {
			validate109.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema74.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate109.errors = vErrors;
		return true;
	}
	validate109.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayMutation = validate110;
	function validate110(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate110.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.action === void 0 && (missing0 = "action")) {
				validate110.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "action" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate110.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.action !== void 0) {
					let data0 = data.action;
					if (typeof data0 !== "string") {
						validate110.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/GatewayAction/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					if (!(data0 === "LAUNCH" || data0 === "PROBE" || data0 === "RESTART" || data0 === "STOP")) {
						validate110.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/GatewayAction/enum",
							keyword: "enum",
							params: { allowedValues: schema74.enum },
							message: "must be equal to one of the allowed values"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.expectedStateVersion !== void 0) {
						let data1 = data.expectedStateVersion;
						if (typeof data1 === "string") {
							if (func1(data1) > 256) {
								validate110.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate110.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate110.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data2 = data.workspaceId;
							if (typeof data2 === "string") {
								if (func1(data2) > 128) {
									validate110.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate110.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate110.errors = [{
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
		} else {
			validate110.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate110.errors = vErrors;
		return true;
	}
	validate110.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayState = validate111;
	function validate111(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate111.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.pinnedVersion === void 0 && (missing0 = "pinnedVersion") || data.endpoint === void 0 && (missing0 = "endpoint") || data.status === void 0 && (missing0 = "status") || data.desiredRunning === void 0 && (missing0 = "desiredRunning") || data.installed === void 0 && (missing0 = "installed") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.discoveredModelCount === void 0 && (missing0 = "discoveredModelCount") || data.restartAttempts === void 0 && (missing0 = "restartAttempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate111.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema43.properties, key0)) {
					validate111.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.desiredRunning !== void 0) {
					if (typeof data.desiredRunning !== "boolean") {
						validate111.errors = [{
							instancePath: instancePath + "/desiredRunning",
							schemaPath: "#/properties/desiredRunning/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.discoveredModelCount !== void 0) {
						let data1 = data.discoveredModelCount;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1))) {
							validate111.errors = [{
								instancePath: instancePath + "/discoveredModelCount",
								schemaPath: "#/properties/discoveredModelCount/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							}];
							return false;
						}
						if (typeof data1 == "number") {
							if (data1 > 1e3 || isNaN(data1)) {
								validate111.errors = [{
									instancePath: instancePath + "/discoveredModelCount",
									schemaPath: "#/properties/discoveredModelCount/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 1e3
									},
									message: "must be <= 1000"
								}];
								return false;
							} else if (data1 < 0 || isNaN(data1)) {
								validate111.errors = [{
									instancePath: instancePath + "/discoveredModelCount",
									schemaPath: "#/properties/discoveredModelCount/minimum",
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
						if (data.endpoint !== void 0) {
							let data2 = data.endpoint;
							if (typeof data2 !== "string") {
								validate111.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if ("http://127.0.0.1:8317" !== data2) {
								validate111.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/const",
									keyword: "const",
									params: { allowedValue: "http://127.0.0.1:8317" },
									message: "must be equal to constant"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.errorCode !== void 0) {
								let data3 = data.errorCode;
								if (typeof data3 !== "string" && data3 !== null) {
									validate111.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema43.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.installed !== void 0) {
									if (typeof data.installed !== "boolean") {
										validate111.errors = [{
											instancePath: instancePath + "/installed",
											schemaPath: "#/properties/installed/type",
											keyword: "type",
											params: { type: "boolean" },
											message: "must be boolean"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.lastProbeAt !== void 0) {
										let data5 = data.lastProbeAt;
										if (typeof data5 !== "string" && data5 !== null) {
											validate111.errors = [{
												instancePath: instancePath + "/lastProbeAt",
												schemaPath: "#/properties/lastProbeAt/type",
												keyword: "type",
												params: { type: schema43.properties.lastProbeAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelAvailable !== void 0) {
											if (typeof data.modelAvailable !== "boolean") {
												validate111.errors = [{
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
										if (valid0) {
											if (data.nextRetryAt !== void 0) {
												let data7 = data.nextRetryAt;
												if (typeof data7 !== "string" && data7 !== null) {
													validate111.errors = [{
														instancePath: instancePath + "/nextRetryAt",
														schemaPath: "#/properties/nextRetryAt/type",
														keyword: "type",
														params: { type: schema43.properties.nextRetryAt.type },
														message: "must be string,null"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.pinnedVersion !== void 0) {
													let data8 = data.pinnedVersion;
													if (typeof data8 !== "string") {
														validate111.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if ("7.2.155" !== data8) {
														validate111.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/const",
															keyword: "const",
															params: { allowedValue: "7.2.155" },
															message: "must be equal to constant"
														}];
														return false;
													}
													var valid0 = true;
												} else var valid0 = true;
												if (valid0) {
													if (data.restartAttempts !== void 0) {
														let data9 = data.restartAttempts;
														if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9))) {
															validate111.errors = [{
																instancePath: instancePath + "/restartAttempts",
																schemaPath: "#/properties/restartAttempts/type",
																keyword: "type",
																params: { type: "integer" },
																message: "must be integer"
															}];
															return false;
														}
														if (typeof data9 == "number") {
															if (data9 > 3 || isNaN(data9)) {
																validate111.errors = [{
																	instancePath: instancePath + "/restartAttempts",
																	schemaPath: "#/properties/restartAttempts/maximum",
																	keyword: "maximum",
																	params: {
																		comparison: "<=",
																		limit: 3
																	},
																	message: "must be <= 3"
																}];
																return false;
															} else if (data9 < 0 || isNaN(data9)) {
																validate111.errors = [{
																	instancePath: instancePath + "/restartAttempts",
																	schemaPath: "#/properties/restartAttempts/minimum",
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
														if (data.stateVersion !== void 0) {
															let data10 = data.stateVersion;
															if (typeof data10 === "string") {
																if (func1(data10) > 256) {
																	validate111.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data10) < 1) {
																	validate111.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate111.errors = [{
																	instancePath: instancePath + "/stateVersion",
																	schemaPath: "#/properties/stateVersion/type",
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
																let data11 = data.status;
																if (typeof data11 !== "string") {
																	validate111.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																if (!(data11 === "STOPPED" || data11 === "INSTALLING" || data11 === "STARTING" || data11 === "RUNNING" || data11 === "PORT_CONFLICT" || data11 === "UNAUTHORIZED" || data11 === "BACKOFF" || data11 === "FAILED" || data11 === "STOPPING")) {
																	validate111.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/enum",
																		keyword: "enum",
																		params: { allowedValues: schema44.enum },
																		message: "must be equal to one of the allowed values"
																	}];
																	return false;
																}
																var valid0 = true;
															} else var valid0 = true;
															if (valid0) {
																if (data.updatedAt !== void 0) {
																	if (typeof data.updatedAt !== "string") {
																		validate111.errors = [{
																			instancePath: instancePath + "/updatedAt",
																			schemaPath: "#/properties/updatedAt/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid0 = true;
																} else var valid0 = true;
																if (valid0) {
																	if (data.workspaceId !== void 0) {
																		let data13 = data.workspaceId;
																		if (typeof data13 === "string") {
																			if (func1(data13) > 128) {
																				validate111.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/maxLength",
																					keyword: "maxLength",
																					params: { limit: 128 },
																					message: "must NOT have more than 128 characters"
																				}];
																				return false;
																			} else if (func1(data13) < 1) {
																				validate111.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate111.errors = [{
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
									}
								}
							}
						}
					}
				}
			}
		} else {
			validate111.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate111.errors = vErrors;
		return true;
	}
	validate111.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayStatus = validate112;
	function validate112(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate112.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate112.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "STOPPED" || data === "INSTALLING" || data === "STARTING" || data === "RUNNING" || data === "PORT_CONFLICT" || data === "UNAUTHORIZED" || data === "BACKOFF" || data === "FAILED" || data === "STOPPING")) {
			validate112.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema44.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate112.errors = vErrors;
		return true;
	}
	validate112.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.HardSafetyRule = validate113;
	function validate113(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate113.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.description === void 0 && (missing0 = "description")) {
				validate113.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "description" || key0 === "id")) {
					validate113.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.description !== void 0) {
					let data0 = data.description;
					if (typeof data0 === "string") {
						if (func1(data0) > 256) {
							validate113.errors = [{
								instancePath: instancePath + "/description",
								schemaPath: "#/properties/description/maxLength",
								keyword: "maxLength",
								params: { limit: 256 },
								message: "must NOT have more than 256 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate113.errors = [{
								instancePath: instancePath + "/description",
								schemaPath: "#/properties/description/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate113.errors = [{
							instancePath: instancePath + "/description",
							schemaPath: "#/properties/description/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.id !== void 0) {
						let data1 = data.id;
						if (typeof data1 === "string") {
							if (func1(data1) > 64) {
								validate113.errors = [{
									instancePath: instancePath + "/id",
									schemaPath: "#/properties/id/maxLength",
									keyword: "maxLength",
									params: { limit: 64 },
									message: "must NOT have more than 64 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate113.errors = [{
									instancePath: instancePath + "/id",
									schemaPath: "#/properties/id/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate113.errors = [{
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
				}
			}
		} else {
			validate113.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate113.errors = vErrors;
		return true;
	}
	validate113.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelAttempt = validate114;
	function validate114(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate114.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.attemptId === void 0 && (missing0 = "attemptId") || data.provider === void 0 && (missing0 = "provider") || data.startedAt === void 0 && (missing0 = "startedAt") || data.endedAt === void 0 && (missing0 = "endedAt") || data.outcome === void 0 && (missing0 = "outcome")) {
					validate114.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema46.properties, key0)) {
						validate114.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.attemptId !== void 0) {
							let data0 = data.attemptId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate114.errors = [{
											instancePath: instancePath + "/attemptId",
											schemaPath: "#/properties/attemptId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate114.errors = [{
											instancePath: instancePath + "/attemptId",
											schemaPath: "#/properties/attemptId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate114.errors = [{
										instancePath: instancePath + "/attemptId",
										schemaPath: "#/properties/attemptId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.endedAt !== void 0) {
								const _errs4 = errors;
								if (typeof data.endedAt !== "string") {
									validate114.errors = [{
										instancePath: instancePath + "/endedAt",
										schemaPath: "#/properties/endedAt/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.errorCategory !== void 0) {
									let data2 = data.errorCategory;
									const _errs6 = errors;
									if (typeof data2 !== "string" && data2 !== null) {
										validate114.errors = [{
											instancePath: instancePath + "/errorCategory",
											schemaPath: "#/properties/errorCategory/type",
											keyword: "type",
											params: { type: schema46.properties.errorCategory.type },
											message: "must be string,null"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.kind !== void 0) {
										let data3 = data.kind;
										const _errs8 = errors;
										if (typeof data3 !== "string") {
											validate114.errors = [{
												instancePath: instancePath + "/kind",
												schemaPath: "#/$defs/ModelAttemptKind/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "SETUP" || data3 === "THREAD")) {
											validate114.errors = [{
												instancePath: instancePath + "/kind",
												schemaPath: "#/$defs/ModelAttemptKind/enum",
												keyword: "enum",
												params: { allowedValues: schema47.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelId !== void 0) {
											let data4 = data.modelId;
											const _errs11 = errors;
											if (typeof data4 !== "string" && data4 !== null) {
												validate114.errors = [{
													instancePath: instancePath + "/modelId",
													schemaPath: "#/properties/modelId/type",
													keyword: "type",
													params: { type: schema46.properties.modelId.type },
													message: "must be string,null"
												}];
												return false;
											}
											var valid0 = _errs11 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.outcome !== void 0) {
												let data5 = data.outcome;
												const _errs13 = errors;
												if (typeof data5 !== "string") {
													validate114.errors = [{
														instancePath: instancePath + "/outcome",
														schemaPath: "#/$defs/ModelAttemptOutcome/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												if (!(data5 === "VERIFIED" || data5 === "CONFIGURED" || data5 === "CANCELLED" || data5 === "FAILED")) {
													validate114.errors = [{
														instancePath: instancePath + "/outcome",
														schemaPath: "#/$defs/ModelAttemptOutcome/enum",
														keyword: "enum",
														params: { allowedValues: schema48.enum },
														message: "must be equal to one of the allowed values"
													}];
													return false;
												}
												var valid0 = _errs13 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.provider !== void 0) {
													let data6 = data.provider;
													const _errs16 = errors;
													if (typeof data6 !== "string") {
														validate114.errors = [{
															instancePath: instancePath + "/provider",
															schemaPath: "#/$defs/ModelProvider/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if (!(data6 === "CHATGPT" || data6 === "DEEPSEEK")) {
														validate114.errors = [{
															instancePath: instancePath + "/provider",
															schemaPath: "#/$defs/ModelProvider/enum",
															keyword: "enum",
															params: { allowedValues: schema49.enum },
															message: "must be equal to one of the allowed values"
														}];
														return false;
													}
													var valid0 = _errs16 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.quota !== void 0) {
														let data7 = data.quota;
														const _errs19 = errors;
														const _errs20 = errors;
														let valid4 = false;
														const _errs21 = errors;
														if (errors === errors) {
															if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
																const _errs24 = errors;
																for (const key1 in data7) if (!(key1 === "remaining" || key1 === "resetAt" || key1 === "retryAfterSeconds" || key1 === "window")) {
																	const err0 = {
																		instancePath: instancePath + "/quota",
																		schemaPath: "#/$defs/ModelQuota/additionalProperties",
																		keyword: "additionalProperties",
																		params: { additionalProperty: key1 },
																		message: "must NOT have additional properties"
																	};
																	if (vErrors === null) vErrors = [err0];
																	else vErrors.push(err0);
																	errors++;
																	break;
																}
																if (_errs24 === errors) {
																	if (data7.remaining !== void 0) {
																		let data8 = data7.remaining;
																		const _errs25 = errors;
																		if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8)) && data8 !== null) {
																			const err1 = {
																				instancePath: instancePath + "/quota/remaining",
																				schemaPath: "#/$defs/ModelQuota/properties/remaining/type",
																				keyword: "type",
																				params: { type: schema50.properties.remaining.type },
																				message: "must be integer,null"
																			};
																			if (vErrors === null) vErrors = [err1];
																			else vErrors.push(err1);
																			errors++;
																		}
																		if (errors === _errs25) {
																			if (typeof data8 == "number") {
																				if (data8 > 1e9 || isNaN(data8)) {
																					const err2 = {
																						instancePath: instancePath + "/quota/remaining",
																						schemaPath: "#/$defs/ModelQuota/properties/remaining/maximum",
																						keyword: "maximum",
																						params: {
																							comparison: "<=",
																							limit: 1e9
																						},
																						message: "must be <= 1000000000"
																					};
																					if (vErrors === null) vErrors = [err2];
																					else vErrors.push(err2);
																					errors++;
																				} else if (data8 < 0 || isNaN(data8)) {
																					const err3 = {
																						instancePath: instancePath + "/quota/remaining",
																						schemaPath: "#/$defs/ModelQuota/properties/remaining/minimum",
																						keyword: "minimum",
																						params: {
																							comparison: ">=",
																							limit: 0
																						},
																						message: "must be >= 0"
																					};
																					if (vErrors === null) vErrors = [err3];
																					else vErrors.push(err3);
																					errors++;
																				}
																			}
																		}
																		var valid6 = _errs25 === errors;
																	} else var valid6 = true;
																	if (valid6) {
																		if (data7.resetAt !== void 0) {
																			let data9 = data7.resetAt;
																			const _errs27 = errors;
																			if (typeof data9 !== "string" && data9 !== null) {
																				const err4 = {
																					instancePath: instancePath + "/quota/resetAt",
																					schemaPath: "#/$defs/ModelQuota/properties/resetAt/type",
																					keyword: "type",
																					params: { type: schema50.properties.resetAt.type },
																					message: "must be string,null"
																				};
																				if (vErrors === null) vErrors = [err4];
																				else vErrors.push(err4);
																				errors++;
																			}
																			var valid6 = _errs27 === errors;
																		} else var valid6 = true;
																		if (valid6) {
																			if (data7.retryAfterSeconds !== void 0) {
																				let data10 = data7.retryAfterSeconds;
																				const _errs29 = errors;
																				if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10)) && data10 !== null) {
																					const err5 = {
																						instancePath: instancePath + "/quota/retryAfterSeconds",
																						schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/type",
																						keyword: "type",
																						params: { type: schema50.properties.retryAfterSeconds.type },
																						message: "must be integer,null"
																					};
																					if (vErrors === null) vErrors = [err5];
																					else vErrors.push(err5);
																					errors++;
																				}
																				if (errors === _errs29) {
																					if (typeof data10 == "number") {
																						if (data10 > 86400 || isNaN(data10)) {
																							const err6 = {
																								instancePath: instancePath + "/quota/retryAfterSeconds",
																								schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/maximum",
																								keyword: "maximum",
																								params: {
																									comparison: "<=",
																									limit: 86400
																								},
																								message: "must be <= 86400"
																							};
																							if (vErrors === null) vErrors = [err6];
																							else vErrors.push(err6);
																							errors++;
																						} else if (data10 < 0 || isNaN(data10)) {
																							const err7 = {
																								instancePath: instancePath + "/quota/retryAfterSeconds",
																								schemaPath: "#/$defs/ModelQuota/properties/retryAfterSeconds/minimum",
																								keyword: "minimum",
																								params: {
																									comparison: ">=",
																									limit: 0
																								},
																								message: "must be >= 0"
																							};
																							if (vErrors === null) vErrors = [err7];
																							else vErrors.push(err7);
																							errors++;
																						}
																					}
																				}
																				var valid6 = _errs29 === errors;
																			} else var valid6 = true;
																			if (valid6) {
																				if (data7.window !== void 0) {
																					let data11 = data7.window;
																					const _errs31 = errors;
																					if (typeof data11 !== "string" && data11 !== null) {
																						const err8 = {
																							instancePath: instancePath + "/quota/window",
																							schemaPath: "#/$defs/ModelQuota/properties/window/type",
																							keyword: "type",
																							params: { type: schema50.properties.window.type },
																							message: "must be string,null"
																						};
																						if (vErrors === null) vErrors = [err8];
																						else vErrors.push(err8);
																						errors++;
																					}
																					var valid6 = _errs31 === errors;
																				} else var valid6 = true;
																			}
																		}
																	}
																}
															} else {
																const err9 = {
																	instancePath: instancePath + "/quota",
																	schemaPath: "#/$defs/ModelQuota/type",
																	keyword: "type",
																	params: { type: "object" },
																	message: "must be object"
																};
																if (vErrors === null) vErrors = [err9];
																else vErrors.push(err9);
																errors++;
															}
														}
														var _valid0 = _errs21 === errors;
														valid4 = valid4 || _valid0;
														const _errs33 = errors;
														if (data7 !== null) {
															const err10 = {
																instancePath: instancePath + "/quota",
																schemaPath: "#/properties/quota/anyOf/1/type",
																keyword: "type",
																params: { type: "null" },
																message: "must be null"
															};
															if (vErrors === null) vErrors = [err10];
															else vErrors.push(err10);
															errors++;
														}
														var _valid0 = _errs33 === errors;
														valid4 = valid4 || _valid0;
														if (!valid4) {
															const err11 = {
																instancePath: instancePath + "/quota",
																schemaPath: "#/properties/quota/anyOf",
																keyword: "anyOf",
																params: {},
																message: "must match a schema in anyOf"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
															validate114.errors = vErrors;
															return false;
														} else {
															errors = _errs20;
															if (vErrors !== null) {
																if (_errs20) vErrors.length = _errs20;
																else vErrors = null;
															}
														}
														var valid0 = _errs19 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.startedAt !== void 0) {
															const _errs35 = errors;
															if (typeof data.startedAt !== "string") {
																validate114.errors = [{
																	instancePath: instancePath + "/startedAt",
																	schemaPath: "#/properties/startedAt/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid0 = _errs35 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.thinkingType !== void 0) {
																let data13 = data.thinkingType;
																const _errs37 = errors;
																const _errs38 = errors;
																let valid7 = false;
																const _errs39 = errors;
																if (typeof data13 !== "string") {
																	const err12 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/$defs/ThinkingType/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	};
																	if (vErrors === null) vErrors = [err12];
																	else vErrors.push(err12);
																	errors++;
																}
																if (!(data13 === "disabled" || data13 === "enabled")) {
																	const err13 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/$defs/ThinkingType/enum",
																		keyword: "enum",
																		params: { allowedValues: schema51.enum },
																		message: "must be equal to one of the allowed values"
																	};
																	if (vErrors === null) vErrors = [err13];
																	else vErrors.push(err13);
																	errors++;
																}
																var _valid1 = _errs39 === errors;
																valid7 = valid7 || _valid1;
																const _errs42 = errors;
																if (data13 !== null) {
																	const err14 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/properties/thinkingType/anyOf/1/type",
																		keyword: "type",
																		params: { type: "null" },
																		message: "must be null"
																	};
																	if (vErrors === null) vErrors = [err14];
																	else vErrors.push(err14);
																	errors++;
																}
																var _valid1 = _errs42 === errors;
																valid7 = valid7 || _valid1;
																if (!valid7) {
																	const err15 = {
																		instancePath: instancePath + "/thinkingType",
																		schemaPath: "#/properties/thinkingType/anyOf",
																		keyword: "anyOf",
																		params: {},
																		message: "must match a schema in anyOf"
																	};
																	if (vErrors === null) vErrors = [err15];
																	else vErrors.push(err15);
																	errors++;
																	validate114.errors = vErrors;
																	return false;
																} else {
																	errors = _errs38;
																	if (vErrors !== null) {
																		if (_errs38) vErrors.length = _errs38;
																		else vErrors = null;
																	}
																}
																var valid0 = _errs37 === errors;
															} else var valid0 = true;
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate114.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate114.errors = vErrors;
		return errors === 0;
	}
	validate114.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelAttemptKind = validate115;
	function validate115(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate115.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate115.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "SETUP" || data === "THREAD")) {
			validate115.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema47.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate115.errors = vErrors;
		return true;
	}
	validate115.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelAttemptOutcome = validate116;
	function validate116(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate116.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate116.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "VERIFIED" || data === "CONFIGURED" || data === "CANCELLED" || data === "FAILED")) {
			validate116.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema48.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate116.errors = vErrors;
		return true;
	}
	validate116.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelHealth = validate117;
	function validate117(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate117.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate117.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "NOT_CONFIGURED" || data === "UNVERIFIED" || data === "VERIFYING" || data === "READY" || data === "FAILED")) {
			validate117.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema57.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate117.errors = vErrors;
		return true;
	}
	validate117.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelProvider = validate118;
	function validate118(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate118.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate118.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "CHATGPT" || data === "DEEPSEEK")) {
			validate118.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema49.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate118.errors = vErrors;
		return true;
	}
	validate118.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelProviderState = validate119;
	function validate119(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate119.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.configured === void 0 && (missing0 = "configured") || data.status === void 0 && (missing0 = "status") || data.routes === void 0 && (missing0 = "routes")) {
					validate119.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "configured" || key0 === "errorCode" || key0 === "lastVerifiedAt" || key0 === "provider" || key0 === "routes" || key0 === "status")) {
						validate119.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.configured !== void 0) {
							const _errs2 = errors;
							if (typeof data.configured !== "boolean") {
								validate119.errors = [{
									instancePath: instancePath + "/configured",
									schemaPath: "#/properties/configured/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.errorCode !== void 0) {
								let data1 = data.errorCode;
								const _errs4 = errors;
								if (typeof data1 !== "string" && data1 !== null) {
									validate119.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema52.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.lastVerifiedAt !== void 0) {
									let data2 = data.lastVerifiedAt;
									const _errs6 = errors;
									if (typeof data2 !== "string" && data2 !== null) {
										validate119.errors = [{
											instancePath: instancePath + "/lastVerifiedAt",
											schemaPath: "#/properties/lastVerifiedAt/type",
											keyword: "type",
											params: { type: schema52.properties.lastVerifiedAt.type },
											message: "must be string,null"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.provider !== void 0) {
										let data3 = data.provider;
										const _errs8 = errors;
										if (typeof data3 !== "string") {
											validate119.errors = [{
												instancePath: instancePath + "/provider",
												schemaPath: "#/$defs/ModelProvider/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "CHATGPT" || data3 === "DEEPSEEK")) {
											validate119.errors = [{
												instancePath: instancePath + "/provider",
												schemaPath: "#/$defs/ModelProvider/enum",
												keyword: "enum",
												params: { allowedValues: schema49.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.routes !== void 0) {
											let data4 = data.routes;
											const _errs11 = errors;
											if (errors === _errs11) {
												if (Array.isArray(data4)) {
													if (data4.length > 100) {
														validate119.errors = [{
															instancePath: instancePath + "/routes",
															schemaPath: "#/properties/routes/maxItems",
															keyword: "maxItems",
															params: { limit: 100 },
															message: "must NOT have more than 100 items"
														}];
														return false;
													} else {
														const len0 = data4.length;
														for (let i0 = 0; i0 < len0; i0++) {
															const _errs13 = errors;
															if (!validate31(data4[i0], {
																instancePath: instancePath + "/routes/" + i0,
																parentData: data4,
																parentDataProperty: i0,
																rootData,
																dynamicAnchors
															})) {
																vErrors = vErrors === null ? validate31.errors : vErrors.concat(validate31.errors);
																errors = vErrors.length;
															}
															if (!(_errs13 === errors)) break;
														}
													}
												} else {
													validate119.errors = [{
														instancePath: instancePath + "/routes",
														schemaPath: "#/properties/routes/type",
														keyword: "type",
														params: { type: "array" },
														message: "must be array"
													}];
													return false;
												}
											}
											var valid0 = _errs11 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.status !== void 0) {
												let data6 = data.status;
												const _errs14 = errors;
												if (typeof data6 !== "string") {
													validate119.errors = [{
														instancePath: instancePath + "/status",
														schemaPath: "#/$defs/ModelHealth/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												if (!(data6 === "NOT_CONFIGURED" || data6 === "UNVERIFIED" || data6 === "VERIFYING" || data6 === "READY" || data6 === "FAILED")) {
													validate119.errors = [{
														instancePath: instancePath + "/status",
														schemaPath: "#/$defs/ModelHealth/enum",
														keyword: "enum",
														params: { allowedValues: schema57.enum },
														message: "must be equal to one of the allowed values"
													}];
													return false;
												}
												var valid0 = _errs14 === errors;
											} else var valid0 = true;
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate119.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate119.errors = vErrors;
		return errors === 0;
	}
	validate119.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelQuery = validate121;
	function validate121(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate121.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId")) {
				validate121.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "workspaceId")) {
					validate121.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.workspaceId !== void 0) {
					let data0 = data.workspaceId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate121.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate121.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate121.errors = [{
							instancePath: instancePath + "/workspaceId",
							schemaPath: "#/properties/workspaceId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
				}
			}
		} else {
			validate121.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate121.errors = vErrors;
		return true;
	}
	validate121.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelQuota = validate122;
	function validate122(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate122.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const key0 in data) if (!(key0 === "remaining" || key0 === "resetAt" || key0 === "retryAfterSeconds" || key0 === "window")) {
				validate122.errors = [{
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				}];
				return false;
			}
			if (data.remaining !== void 0) {
				let data0 = data.remaining;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0)) && data0 !== null) {
					validate122.errors = [{
						instancePath: instancePath + "/remaining",
						schemaPath: "#/properties/remaining/type",
						keyword: "type",
						params: { type: schema50.properties.remaining.type },
						message: "must be integer,null"
					}];
					return false;
				}
				if (typeof data0 == "number") {
					if (data0 > 1e9 || isNaN(data0)) {
						validate122.errors = [{
							instancePath: instancePath + "/remaining",
							schemaPath: "#/properties/remaining/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 1e9
							},
							message: "must be <= 1000000000"
						}];
						return false;
					} else if (data0 < 0 || isNaN(data0)) {
						validate122.errors = [{
							instancePath: instancePath + "/remaining",
							schemaPath: "#/properties/remaining/minimum",
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
				if (data.resetAt !== void 0) {
					let data1 = data.resetAt;
					if (typeof data1 !== "string" && data1 !== null) {
						validate122.errors = [{
							instancePath: instancePath + "/resetAt",
							schemaPath: "#/properties/resetAt/type",
							keyword: "type",
							params: { type: schema50.properties.resetAt.type },
							message: "must be string,null"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.retryAfterSeconds !== void 0) {
						let data2 = data.retryAfterSeconds;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2)) && data2 !== null) {
							validate122.errors = [{
								instancePath: instancePath + "/retryAfterSeconds",
								schemaPath: "#/properties/retryAfterSeconds/type",
								keyword: "type",
								params: { type: schema50.properties.retryAfterSeconds.type },
								message: "must be integer,null"
							}];
							return false;
						}
						if (typeof data2 == "number") {
							if (data2 > 86400 || isNaN(data2)) {
								validate122.errors = [{
									instancePath: instancePath + "/retryAfterSeconds",
									schemaPath: "#/properties/retryAfterSeconds/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 86400
									},
									message: "must be <= 86400"
								}];
								return false;
							} else if (data2 < 0 || isNaN(data2)) {
								validate122.errors = [{
									instancePath: instancePath + "/retryAfterSeconds",
									schemaPath: "#/properties/retryAfterSeconds/minimum",
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
						if (data.window !== void 0) {
							let data3 = data.window;
							if (typeof data3 !== "string" && data3 !== null) {
								validate122.errors = [{
									instancePath: instancePath + "/window",
									schemaPath: "#/properties/window/type",
									keyword: "type",
									params: { type: schema50.properties.window.type },
									message: "must be string,null"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
					}
				}
			}
		} else {
			validate122.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate122.errors = vErrors;
		return true;
	}
	validate122.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelRoute = validate123;
	function validate123(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate123.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate123.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "modelId" || key0 === "provider" || key0 === "thinkingType" || key0 === "verifiedAt")) {
						validate123.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.modelId !== void 0) {
							let data0 = data.modelId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate123.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate123.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate123.errors = [{
										instancePath: instancePath + "/modelId",
										schemaPath: "#/properties/modelId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.provider !== void 0) {
								let data1 = data.provider;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate123.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CHATGPT" || data1 === "DEEPSEEK")) {
									validate123.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/enum",
										keyword: "enum",
										params: { allowedValues: schema49.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.thinkingType !== void 0) {
									let data2 = data.thinkingType;
									const _errs7 = errors;
									const _errs8 = errors;
									let valid2 = false;
									const _errs9 = errors;
									if (typeof data2 !== "string") {
										const err0 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err0];
										else vErrors.push(err0);
										errors++;
									}
									if (!(data2 === "disabled" || data2 === "enabled")) {
										const err1 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/enum",
											keyword: "enum",
											params: { allowedValues: schema51.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err1];
										else vErrors.push(err1);
										errors++;
									}
									var _valid0 = _errs9 === errors;
									valid2 = valid2 || _valid0;
									const _errs12 = errors;
									if (data2 !== null) {
										const err2 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf/1/type",
											keyword: "type",
											params: { type: "null" },
											message: "must be null"
										};
										if (vErrors === null) vErrors = [err2];
										else vErrors.push(err2);
										errors++;
									}
									var _valid0 = _errs12 === errors;
									valid2 = valid2 || _valid0;
									if (!valid2) {
										const err3 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										if (vErrors === null) vErrors = [err3];
										else vErrors.push(err3);
										errors++;
										validate123.errors = vErrors;
										return false;
									} else {
										errors = _errs8;
										if (vErrors !== null) {
											if (_errs8) vErrors.length = _errs8;
											else vErrors = null;
										}
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.verifiedAt !== void 0) {
										let data3 = data.verifiedAt;
										const _errs14 = errors;
										if (typeof data3 !== "string" && data3 !== null) {
											validate123.errors = [{
												instancePath: instancePath + "/verifiedAt",
												schemaPath: "#/properties/verifiedAt/type",
												keyword: "type",
												params: { type: schema54.properties.verifiedAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = _errs14 === errors;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			} else {
				validate123.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate123.errors = vErrors;
		return errors === 0;
	}
	validate123.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelSelection = validate124;
	function validate124(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate124.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate124.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "modelId" || key0 === "provider" || key0 === "thinkingType")) {
						validate124.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.modelId !== void 0) {
							let data0 = data.modelId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 128) {
										validate124.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/maxLength",
											keyword: "maxLength",
											params: { limit: 128 },
											message: "must NOT have more than 128 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate124.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate124.errors = [{
										instancePath: instancePath + "/modelId",
										schemaPath: "#/properties/modelId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.provider !== void 0) {
								let data1 = data.provider;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate124.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CHATGPT" || data1 === "DEEPSEEK")) {
									validate124.errors = [{
										instancePath: instancePath + "/provider",
										schemaPath: "#/$defs/ModelProvider/enum",
										keyword: "enum",
										params: { allowedValues: schema49.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.thinkingType !== void 0) {
									let data2 = data.thinkingType;
									const _errs7 = errors;
									const _errs8 = errors;
									let valid2 = false;
									const _errs9 = errors;
									if (typeof data2 !== "string") {
										const err0 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err0];
										else vErrors.push(err0);
										errors++;
									}
									if (!(data2 === "disabled" || data2 === "enabled")) {
										const err1 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/$defs/ThinkingType/enum",
											keyword: "enum",
											params: { allowedValues: schema51.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err1];
										else vErrors.push(err1);
										errors++;
									}
									var _valid0 = _errs9 === errors;
									valid2 = valid2 || _valid0;
									const _errs12 = errors;
									if (data2 !== null) {
										const err2 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf/1/type",
											keyword: "type",
											params: { type: "null" },
											message: "must be null"
										};
										if (vErrors === null) vErrors = [err2];
										else vErrors.push(err2);
										errors++;
									}
									var _valid0 = _errs12 === errors;
									valid2 = valid2 || _valid0;
									if (!valid2) {
										const err3 = {
											instancePath: instancePath + "/thinkingType",
											schemaPath: "#/properties/thinkingType/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										if (vErrors === null) vErrors = [err3];
										else vErrors.push(err3);
										errors++;
										validate124.errors = vErrors;
										return false;
									} else {
										errors = _errs8;
										if (vErrors !== null) {
											if (_errs8) vErrors.length = _errs8;
											else vErrors = null;
										}
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
							}
						}
					}
				}
			} else {
				validate124.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate124.errors = vErrors;
		return errors === 0;
	}
	validate124.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ModelState = validate125;
	function validate125(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate125.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.chatgpt === void 0 && (missing0 = "chatgpt") || data.deepseek === void 0 && (missing0 = "deepseek") || data.attempts === void 0 && (missing0 = "attempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
					validate125.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func31.call(schema45.properties, key0)) {
						validate125.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.attempts !== void 0) {
							let data0 = data.attempts;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (Array.isArray(data0)) {
									if (data0.length > 100) {
										validate125.errors = [{
											instancePath: instancePath + "/attempts",
											schemaPath: "#/properties/attempts/maxItems",
											keyword: "maxItems",
											params: { limit: 100 },
											message: "must NOT have more than 100 items"
										}];
										return false;
									} else {
										const len0 = data0.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs4 = errors;
											if (!validate28(data0[i0], {
												instancePath: instancePath + "/attempts/" + i0,
												parentData: data0,
												parentDataProperty: i0,
												rootData,
												dynamicAnchors
											})) {
												vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
												errors = vErrors.length;
											}
											if (!(_errs4 === errors)) break;
										}
									}
								} else {
									validate125.errors = [{
										instancePath: instancePath + "/attempts",
										schemaPath: "#/properties/attempts/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.automaticFallback !== void 0) {
								const _errs5 = errors;
								if (typeof data.automaticFallback !== "boolean") {
									validate125.errors = [{
										instancePath: instancePath + "/automaticFallback",
										schemaPath: "#/properties/automaticFallback/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								var valid0 = _errs5 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.chatgpt !== void 0) {
									const _errs7 = errors;
									if (!validate30(data.chatgpt, {
										instancePath: instancePath + "/chatgpt",
										parentData: data,
										parentDataProperty: "chatgpt",
										rootData,
										dynamicAnchors
									})) {
										vErrors = vErrors === null ? validate30.errors : vErrors.concat(validate30.errors);
										errors = vErrors.length;
									}
									var valid0 = _errs7 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.currentRoute !== void 0) {
										let data4 = data.currentRoute;
										const _errs8 = errors;
										const _errs9 = errors;
										let valid2 = false;
										const _errs10 = errors;
										if (!validate31(data4, {
											instancePath: instancePath + "/currentRoute",
											parentData: data,
											parentDataProperty: "currentRoute",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate31.errors : vErrors.concat(validate31.errors);
											errors = vErrors.length;
										}
										var _valid0 = _errs10 === errors;
										valid2 = valid2 || _valid0;
										const _errs11 = errors;
										if (data4 !== null) {
											const err0 = {
												instancePath: instancePath + "/currentRoute",
												schemaPath: "#/properties/currentRoute/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err1 = {
												instancePath: instancePath + "/currentRoute",
												schemaPath: "#/properties/currentRoute/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
											validate125.errors = vErrors;
											return false;
										} else {
											errors = _errs9;
											if (vErrors !== null) {
												if (_errs9) vErrors.length = _errs9;
												else vErrors = null;
											}
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.deepseek !== void 0) {
											const _errs13 = errors;
											if (!validate30(data.deepseek, {
												instancePath: instancePath + "/deepseek",
												parentData: data,
												parentDataProperty: "deepseek",
												rootData,
												dynamicAnchors
											})) {
												vErrors = vErrors === null ? validate30.errors : vErrors.concat(validate30.errors);
												errors = vErrors.length;
											}
											var valid0 = _errs13 === errors;
										} else var valid0 = true;
										if (valid0) {
											if (data.defaultRoute !== void 0) {
												let data6 = data.defaultRoute;
												const _errs14 = errors;
												const _errs15 = errors;
												let valid3 = false;
												const _errs16 = errors;
												if (!validate36(data6, {
													instancePath: instancePath + "/defaultRoute",
													parentData: data,
													parentDataProperty: "defaultRoute",
													rootData,
													dynamicAnchors
												})) {
													vErrors = vErrors === null ? validate36.errors : vErrors.concat(validate36.errors);
													errors = vErrors.length;
												}
												var _valid1 = _errs16 === errors;
												valid3 = valid3 || _valid1;
												const _errs17 = errors;
												if (data6 !== null) {
													const err2 = {
														instancePath: instancePath + "/defaultRoute",
														schemaPath: "#/properties/defaultRoute/anyOf/1/type",
														keyword: "type",
														params: { type: "null" },
														message: "must be null"
													};
													if (vErrors === null) vErrors = [err2];
													else vErrors.push(err2);
													errors++;
												}
												var _valid1 = _errs17 === errors;
												valid3 = valid3 || _valid1;
												if (!valid3) {
													const err3 = {
														instancePath: instancePath + "/defaultRoute",
														schemaPath: "#/properties/defaultRoute/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													if (vErrors === null) vErrors = [err3];
													else vErrors.push(err3);
													errors++;
													validate125.errors = vErrors;
													return false;
												} else {
													errors = _errs15;
													if (vErrors !== null) {
														if (_errs15) vErrors.length = _errs15;
														else vErrors = null;
													}
												}
												var valid0 = _errs14 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.fallbackPolicyVersion !== void 0) {
													let data7 = data.fallbackPolicyVersion;
													const _errs19 = errors;
													if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
														validate125.errors = [{
															instancePath: instancePath + "/fallbackPolicyVersion",
															schemaPath: "#/properties/fallbackPolicyVersion/type",
															keyword: "type",
															params: { type: "integer" },
															message: "must be integer"
														}];
														return false;
													}
													if (errors === _errs19) {
														if (typeof data7 == "number") {
															if (data7 < 1 || isNaN(data7)) {
																validate125.errors = [{
																	instancePath: instancePath + "/fallbackPolicyVersion",
																	schemaPath: "#/properties/fallbackPolicyVersion/minimum",
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
													}
													var valid0 = _errs19 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.stateVersion !== void 0) {
														let data8 = data.stateVersion;
														const _errs21 = errors;
														if (errors === _errs21) {
															if (typeof data8 === "string") {
																if (func1(data8) > 256) {
																	validate125.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data8) < 1) {
																	validate125.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate125.errors = [{
																	instancePath: instancePath + "/stateVersion",
																	schemaPath: "#/properties/stateVersion/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
														}
														var valid0 = _errs21 === errors;
													} else var valid0 = true;
													if (valid0) {
														if (data.updatedAt !== void 0) {
															const _errs23 = errors;
															if (typeof data.updatedAt !== "string") {
																validate125.errors = [{
																	instancePath: instancePath + "/updatedAt",
																	schemaPath: "#/properties/updatedAt/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid0 = _errs23 === errors;
														} else var valid0 = true;
														if (valid0) {
															if (data.workspaceId !== void 0) {
																let data10 = data.workspaceId;
																const _errs25 = errors;
																if (errors === _errs25) {
																	if (typeof data10 === "string") {
																		if (func1(data10) > 128) {
																			validate125.errors = [{
																				instancePath: instancePath + "/workspaceId",
																				schemaPath: "#/properties/workspaceId/maxLength",
																				keyword: "maxLength",
																				params: { limit: 128 },
																				message: "must NOT have more than 128 characters"
																			}];
																			return false;
																		} else if (func1(data10) < 1) {
																			validate125.errors = [{
																				instancePath: instancePath + "/workspaceId",
																				schemaPath: "#/properties/workspaceId/minLength",
																				keyword: "minLength",
																				params: { limit: 1 },
																				message: "must NOT have fewer than 1 characters"
																			}];
																			return false;
																		}
																	} else {
																		validate125.errors = [{
																			instancePath: instancePath + "/workspaceId",
																			schemaPath: "#/properties/workspaceId/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																}
																var valid0 = _errs25 === errors;
															} else var valid0 = true;
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				validate125.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate125.errors = vErrors;
		return errors === 0;
	}
	validate125.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.OpenOrder = validate131;
	function validate131(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate131.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.brokerOrderId === void 0 && (missing0 = "brokerOrderId") || data.symbol === void 0 && (missing0 = "symbol") || data.side === void 0 && (missing0 = "side") || data.status === void 0 && (missing0 = "status")) {
				validate131.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema66.properties, key0)) {
					validate131.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.brokerOrderId !== void 0) {
					if (typeof data.brokerOrderId !== "string") {
						validate131.errors = [{
							instancePath: instancePath + "/brokerOrderId",
							schemaPath: "#/properties/brokerOrderId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.currency !== void 0) {
						let data1 = data.currency;
						if (typeof data1 !== "string" && data1 !== null) {
							validate131.errors = [{
								instancePath: instancePath + "/currency",
								schemaPath: "#/properties/currency/type",
								keyword: "type",
								params: { type: schema66.properties.currency.type },
								message: "must be string,null"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.filledQuantity !== void 0) {
							let data2 = data.filledQuantity;
							if (typeof data2 !== "string" && data2 !== null) {
								validate131.errors = [{
									instancePath: instancePath + "/filledQuantity",
									schemaPath: "#/properties/filledQuantity/type",
									keyword: "type",
									params: { type: schema66.properties.filledQuantity.type },
									message: "must be string,null"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.filledValue !== void 0) {
								let data3 = data.filledValue;
								if (typeof data3 !== "string" && data3 !== null) {
									validate131.errors = [{
										instancePath: instancePath + "/filledValue",
										schemaPath: "#/properties/filledValue/type",
										keyword: "type",
										params: { type: schema66.properties.filledValue.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.kind !== void 0) {
									let data4 = data.kind;
									if (typeof data4 !== "string" && data4 !== null) {
										validate131.errors = [{
											instancePath: instancePath + "/kind",
											schemaPath: "#/properties/kind/type",
											keyword: "type",
											params: { type: schema66.properties.kind.type },
											message: "must be string,null"
										}];
										return false;
									}
									if (!(data4 === "NORMAL" || data4 === "TPSL" || data4 === "PLAN" || data4 === null)) {
										validate131.errors = [{
											instancePath: instancePath + "/kind",
											schemaPath: "#/properties/kind/enum",
											keyword: "enum",
											params: { allowedValues: schema66.properties.kind.enum },
											message: "must be equal to one of the allowed values"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.limitPrice !== void 0) {
										let data5 = data.limitPrice;
										if (typeof data5 !== "string" && data5 !== null) {
											validate131.errors = [{
												instancePath: instancePath + "/limitPrice",
												schemaPath: "#/properties/limitPrice/type",
												keyword: "type",
												params: { type: schema66.properties.limitPrice.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.notional !== void 0) {
											let data6 = data.notional;
											if (typeof data6 !== "string" && data6 !== null) {
												validate131.errors = [{
													instancePath: instancePath + "/notional",
													schemaPath: "#/properties/notional/type",
													keyword: "type",
													params: { type: schema66.properties.notional.type },
													message: "must be string,null"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.quantity !== void 0) {
												let data7 = data.quantity;
												if (typeof data7 !== "string" && data7 !== null) {
													validate131.errors = [{
														instancePath: instancePath + "/quantity",
														schemaPath: "#/properties/quantity/type",
														keyword: "type",
														params: { type: schema66.properties.quantity.type },
														message: "must be string,null"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.side !== void 0) {
													if (typeof data.side !== "string") {
														validate131.errors = [{
															instancePath: instancePath + "/side",
															schemaPath: "#/properties/side/type",
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
															validate131.errors = [{
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
													if (valid0) {
														if (data.symbol !== void 0) {
															if (typeof data.symbol !== "string") {
																validate131.errors = [{
																	instancePath: instancePath + "/symbol",
																	schemaPath: "#/properties/symbol/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid0 = true;
														} else var valid0 = true;
														if (valid0) {
															if (data.triggerPrice !== void 0) {
																let data11 = data.triggerPrice;
																if (typeof data11 !== "string" && data11 !== null) {
																	validate131.errors = [{
																		instancePath: instancePath + "/triggerPrice",
																		schemaPath: "#/properties/triggerPrice/type",
																		keyword: "type",
																		params: { type: schema66.properties.triggerPrice.type },
																		message: "must be string,null"
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
							}
						}
					}
				}
			}
		} else {
			validate131.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate131.errors = vErrors;
		return true;
	}
	validate131.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.OpenWorkspace = validate132;
	var pattern4 = /* @__PURE__ */ new RegExp("^[A-Z]{3}$", "u");
	function validate132(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate132.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "name" || key0 === "path")) {
				validate132.errors = [{
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
						validate132.errors = [{
							instancePath: instancePath + "/baseCurrency",
							schemaPath: "#/properties/baseCurrency/pattern",
							keyword: "pattern",
							params: { pattern: "^[A-Z]{3}$" },
							message: "must match pattern \"^[A-Z]{3}$\""
						}];
						return false;
					}
				} else {
					validate132.errors = [{
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
							validate132.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/maxLength",
								keyword: "maxLength",
								params: { limit: 120 },
								message: "must NOT have more than 120 characters"
							}];
							return false;
						} else if (func1(data1) < 1) {
							validate132.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate132.errors = [{
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
							validate132.errors = [{
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
			validate132.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate132.errors = vErrors;
		return true;
	}
	validate132.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.PermissionReview = validate133;
	function validate133(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate133.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.scope === void 0 && (missing0 = "scope") || data.detected === void 0 && (missing0 = "detected") || data.forbidden === void 0 && (missing0 = "forbidden") || data.unsupported === void 0 && (missing0 = "unsupported") || data.acknowledged === void 0 && (missing0 = "acknowledged") || data.ipAllowListStatus === void 0 && (missing0 = "ipAllowListStatus")) {
				validate133.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "acknowledged" || key0 === "detected" || key0 === "forbidden" || key0 === "ipAllowList" || key0 === "ipAllowListStatus" || key0 === "scope" || key0 === "unsupported")) {
					validate133.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.acknowledged !== void 0) {
					if (typeof data.acknowledged !== "boolean") {
						validate133.errors = [{
							instancePath: instancePath + "/acknowledged",
							schemaPath: "#/properties/acknowledged/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.detected !== void 0) {
						let data1 = data.detected;
						if (Array.isArray(data1)) {
							const len0 = data1.length;
							for (let i0 = 0; i0 < len0; i0++) if (typeof data1[i0] !== "string") {
								validate133.errors = [{
									instancePath: instancePath + "/detected/" + i0,
									schemaPath: "#/properties/detected/items/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
						} else {
							validate133.errors = [{
								instancePath: instancePath + "/detected",
								schemaPath: "#/properties/detected/type",
								keyword: "type",
								params: { type: "array" },
								message: "must be array"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.forbidden !== void 0) {
							let data3 = data.forbidden;
							if (Array.isArray(data3)) {
								const len1 = data3.length;
								for (let i1 = 0; i1 < len1; i1++) if (typeof data3[i1] !== "string") {
									validate133.errors = [{
										instancePath: instancePath + "/forbidden/" + i1,
										schemaPath: "#/properties/forbidden/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate133.errors = [{
									instancePath: instancePath + "/forbidden",
									schemaPath: "#/properties/forbidden/type",
									keyword: "type",
									params: { type: "array" },
									message: "must be array"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.ipAllowList !== void 0) {
								let data5 = data.ipAllowList;
								if (!Array.isArray(data5) && data5 !== null) {
									validate133.errors = [{
										instancePath: instancePath + "/ipAllowList",
										schemaPath: "#/properties/ipAllowList/type",
										keyword: "type",
										params: { type: schema69.properties.ipAllowList.type },
										message: "must be array,null"
									}];
									return false;
								}
								if (Array.isArray(data5)) {
									const len2 = data5.length;
									for (let i2 = 0; i2 < len2; i2++) if (typeof data5[i2] !== "string") {
										validate133.errors = [{
											instancePath: instancePath + "/ipAllowList/" + i2,
											schemaPath: "#/properties/ipAllowList/items/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.ipAllowListStatus !== void 0) {
									if (typeof data.ipAllowListStatus !== "string") {
										validate133.errors = [{
											instancePath: instancePath + "/ipAllowListStatus",
											schemaPath: "#/properties/ipAllowListStatus/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.scope !== void 0) {
										let data8 = data.scope;
										if (typeof data8 !== "string") {
											validate133.errors = [{
												instancePath: instancePath + "/scope",
												schemaPath: "#/properties/scope/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data8 === "VERIFIED" || data8 === "UNVERIFIED")) {
											validate133.errors = [{
												instancePath: instancePath + "/scope",
												schemaPath: "#/properties/scope/enum",
												keyword: "enum",
												params: { allowedValues: schema69.properties.scope.enum },
												message: "must be equal to one of the allowed values"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.unsupported !== void 0) {
											let data9 = data.unsupported;
											if (Array.isArray(data9)) {
												const len3 = data9.length;
												for (let i3 = 0; i3 < len3; i3++) if (typeof data9[i3] !== "string") {
													validate133.errors = [{
														instancePath: instancePath + "/unsupported/" + i3,
														schemaPath: "#/properties/unsupported/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate133.errors = [{
													instancePath: instancePath + "/unsupported",
													schemaPath: "#/properties/unsupported/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
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
			validate133.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate133.errors = vErrors;
		return true;
	}
	validate133.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Position = validate134;
	function validate134(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate134.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.symbol === void 0 && (missing0 = "symbol") || data.quantity === void 0 && (missing0 = "quantity")) {
				validate134.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "averageEntryPrice" || key0 === "instrumentCurrency" || key0 === "marketValue" || key0 === "marketValueCurrency" || key0 === "quantity" || key0 === "symbol")) {
					validate134.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.averageEntryPrice !== void 0) {
					let data0 = data.averageEntryPrice;
					if (typeof data0 !== "string" && data0 !== null) {
						validate134.errors = [{
							instancePath: instancePath + "/averageEntryPrice",
							schemaPath: "#/properties/averageEntryPrice/type",
							keyword: "type",
							params: { type: schema67.properties.averageEntryPrice.type },
							message: "must be string,null"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.instrumentCurrency !== void 0) {
						let data1 = data.instrumentCurrency;
						if (typeof data1 !== "string" && data1 !== null) {
							validate134.errors = [{
								instancePath: instancePath + "/instrumentCurrency",
								schemaPath: "#/properties/instrumentCurrency/type",
								keyword: "type",
								params: { type: schema67.properties.instrumentCurrency.type },
								message: "must be string,null"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.marketValue !== void 0) {
							let data2 = data.marketValue;
							if (typeof data2 !== "string" && data2 !== null) {
								validate134.errors = [{
									instancePath: instancePath + "/marketValue",
									schemaPath: "#/properties/marketValue/type",
									keyword: "type",
									params: { type: schema67.properties.marketValue.type },
									message: "must be string,null"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.marketValueCurrency !== void 0) {
								let data3 = data.marketValueCurrency;
								if (typeof data3 !== "string" && data3 !== null) {
									validate134.errors = [{
										instancePath: instancePath + "/marketValueCurrency",
										schemaPath: "#/properties/marketValueCurrency/type",
										keyword: "type",
										params: { type: schema67.properties.marketValueCurrency.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.quantity !== void 0) {
									if (typeof data.quantity !== "string") {
										validate134.errors = [{
											instancePath: instancePath + "/quantity",
											schemaPath: "#/properties/quantity/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.symbol !== void 0) {
										if (typeof data.symbol !== "string") {
											validate134.errors = [{
												instancePath: instancePath + "/symbol",
												schemaPath: "#/properties/symbol/type",
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
		} else {
			validate134.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate134.errors = vErrors;
		return true;
	}
	validate134.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderCatalog = validate135;
	var schema87 = {
		"type": "object",
		"properties": {
			"available": { "type": "boolean" },
			"displayName": { "type": "string" },
			"environment": { "type": "string" },
			"fields": {
				"type": "array",
				"items": { "$ref": "#/$defs/ProviderField" }
			},
			"forbiddenPermissions": {
				"type": "array",
				"items": { "type": "string" }
			},
			"helpText": { "type": "string" },
			"optionalPermissions": {
				"type": "array",
				"items": { "type": "string" }
			},
			"providerId": { "type": "string" },
			"requiredPermissions": {
				"type": "array",
				"items": { "type": "string" }
			}
		},
		"additionalProperties": false,
		"required": [
			"providerId",
			"displayName",
			"environment",
			"available",
			"helpText",
			"fields",
			"requiredPermissions",
			"optionalPermissions",
			"forbiddenPermissions"
		]
	};
	function validate61(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate61.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.displayName === void 0 && (missing0 = "displayName") || data.environment === void 0 && (missing0 = "environment") || data.available === void 0 && (missing0 = "available") || data.helpText === void 0 && (missing0 = "helpText") || data.fields === void 0 && (missing0 = "fields") || data.requiredPermissions === void 0 && (missing0 = "requiredPermissions") || data.optionalPermissions === void 0 && (missing0 = "optionalPermissions") || data.forbiddenPermissions === void 0 && (missing0 = "forbiddenPermissions")) {
				validate61.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema87.properties, key0)) {
					validate61.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.available !== void 0) {
					if (typeof data.available !== "boolean") {
						validate61.errors = [{
							instancePath: instancePath + "/available",
							schemaPath: "#/properties/available/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.displayName !== void 0) {
						if (typeof data.displayName !== "string") {
							validate61.errors = [{
								instancePath: instancePath + "/displayName",
								schemaPath: "#/properties/displayName/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.environment !== void 0) {
							if (typeof data.environment !== "string") {
								validate61.errors = [{
									instancePath: instancePath + "/environment",
									schemaPath: "#/properties/environment/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.fields !== void 0) {
								let data3 = data.fields;
								if (Array.isArray(data3)) {
									const len0 = data3.length;
									for (let i0 = 0; i0 < len0; i0++) {
										let data4 = data3[i0];
										if (data4 && typeof data4 == "object" && !Array.isArray(data4)) {
											let missing1;
											if (data4.id === void 0 && (missing1 = "id") || data4.label === void 0 && (missing1 = "label") || data4.inputType === void 0 && (missing1 = "inputType") || data4.required === void 0 && (missing1 = "required") || data4.secret === void 0 && (missing1 = "secret") || data4.maxLength === void 0 && (missing1 = "maxLength") || data4.helpText === void 0 && (missing1 = "helpText") || data4.environment === void 0 && (missing1 = "environment")) {
												validate61.errors = [{
													instancePath: instancePath + "/fields/" + i0,
													schemaPath: "#/$defs/ProviderField/required",
													keyword: "required",
													params: { missingProperty: missing1 },
													message: "must have required property '" + missing1 + "'"
												}];
												return false;
											} else {
												for (const key1 in data4) if (!(key1 === "environment" || key1 === "helpText" || key1 === "id" || key1 === "inputType" || key1 === "label" || key1 === "maxLength" || key1 === "required" || key1 === "secret")) {
													validate61.errors = [{
														instancePath: instancePath + "/fields/" + i0,
														schemaPath: "#/$defs/ProviderField/additionalProperties",
														keyword: "additionalProperties",
														params: { additionalProperty: key1 },
														message: "must NOT have additional properties"
													}];
													return false;
												}
												if (data4.environment !== void 0) {
													if (typeof data4.environment !== "string") {
														validate61.errors = [{
															instancePath: instancePath + "/fields/" + i0 + "/environment",
															schemaPath: "#/$defs/ProviderField/properties/environment/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid3 = true;
												} else var valid3 = true;
												if (valid3) {
													if (data4.helpText !== void 0) {
														if (typeof data4.helpText !== "string") {
															validate61.errors = [{
																instancePath: instancePath + "/fields/" + i0 + "/helpText",
																schemaPath: "#/$defs/ProviderField/properties/helpText/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data4.id !== void 0) {
															if (typeof data4.id !== "string") {
																validate61.errors = [{
																	instancePath: instancePath + "/fields/" + i0 + "/id",
																	schemaPath: "#/$defs/ProviderField/properties/id/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
														if (valid3) {
															if (data4.inputType !== void 0) {
																if (typeof data4.inputType !== "string") {
																	validate61.errors = [{
																		instancePath: instancePath + "/fields/" + i0 + "/inputType",
																		schemaPath: "#/$defs/ProviderField/properties/inputType/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																var valid3 = true;
															} else var valid3 = true;
															if (valid3) {
																if (data4.label !== void 0) {
																	if (typeof data4.label !== "string") {
																		validate61.errors = [{
																			instancePath: instancePath + "/fields/" + i0 + "/label",
																			schemaPath: "#/$defs/ProviderField/properties/label/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid3 = true;
																} else var valid3 = true;
																if (valid3) {
																	if (data4.maxLength !== void 0) {
																		let data10 = data4.maxLength;
																		if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
																			validate61.errors = [{
																				instancePath: instancePath + "/fields/" + i0 + "/maxLength",
																				schemaPath: "#/$defs/ProviderField/properties/maxLength/type",
																				keyword: "type",
																				params: { type: "integer" },
																				message: "must be integer"
																			}];
																			return false;
																		}
																		if (typeof data10 == "number") {
																			if (data10 < 0 || isNaN(data10)) {
																				validate61.errors = [{
																					instancePath: instancePath + "/fields/" + i0 + "/maxLength",
																					schemaPath: "#/$defs/ProviderField/properties/maxLength/minimum",
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
																		var valid3 = true;
																	} else var valid3 = true;
																	if (valid3) {
																		if (data4.required !== void 0) {
																			if (typeof data4.required !== "boolean") {
																				validate61.errors = [{
																					instancePath: instancePath + "/fields/" + i0 + "/required",
																					schemaPath: "#/$defs/ProviderField/properties/required/type",
																					keyword: "type",
																					params: { type: "boolean" },
																					message: "must be boolean"
																				}];
																				return false;
																			}
																			var valid3 = true;
																		} else var valid3 = true;
																		if (valid3) {
																			if (data4.secret !== void 0) {
																				if (typeof data4.secret !== "boolean") {
																					validate61.errors = [{
																						instancePath: instancePath + "/fields/" + i0 + "/secret",
																						schemaPath: "#/$defs/ProviderField/properties/secret/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					}];
																					return false;
																				}
																				var valid3 = true;
																			} else var valid3 = true;
																		}
																	}
																}
															}
														}
													}
												}
											}
										} else {
											validate61.errors = [{
												instancePath: instancePath + "/fields/" + i0,
												schemaPath: "#/$defs/ProviderField/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											}];
											return false;
										}
									}
								} else {
									validate61.errors = [{
										instancePath: instancePath + "/fields",
										schemaPath: "#/properties/fields/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.forbiddenPermissions !== void 0) {
									let data13 = data.forbiddenPermissions;
									if (Array.isArray(data13)) {
										const len1 = data13.length;
										for (let i1 = 0; i1 < len1; i1++) if (typeof data13[i1] !== "string") {
											validate61.errors = [{
												instancePath: instancePath + "/forbiddenPermissions/" + i1,
												schemaPath: "#/properties/forbiddenPermissions/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate61.errors = [{
											instancePath: instancePath + "/forbiddenPermissions",
											schemaPath: "#/properties/forbiddenPermissions/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.helpText !== void 0) {
										if (typeof data.helpText !== "string") {
											validate61.errors = [{
												instancePath: instancePath + "/helpText",
												schemaPath: "#/properties/helpText/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.optionalPermissions !== void 0) {
											let data16 = data.optionalPermissions;
											if (Array.isArray(data16)) {
												const len2 = data16.length;
												for (let i2 = 0; i2 < len2; i2++) if (typeof data16[i2] !== "string") {
													validate61.errors = [{
														instancePath: instancePath + "/optionalPermissions/" + i2,
														schemaPath: "#/properties/optionalPermissions/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate61.errors = [{
													instancePath: instancePath + "/optionalPermissions",
													schemaPath: "#/properties/optionalPermissions/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.providerId !== void 0) {
												if (typeof data.providerId !== "string") {
													validate61.errors = [{
														instancePath: instancePath + "/providerId",
														schemaPath: "#/properties/providerId/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.requiredPermissions !== void 0) {
													let data19 = data.requiredPermissions;
													if (Array.isArray(data19)) {
														const len3 = data19.length;
														for (let i3 = 0; i3 < len3; i3++) if (typeof data19[i3] !== "string") {
															validate61.errors = [{
																instancePath: instancePath + "/requiredPermissions/" + i3,
																schemaPath: "#/properties/requiredPermissions/items/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
													} else {
														validate61.errors = [{
															instancePath: instancePath + "/requiredPermissions",
															schemaPath: "#/properties/requiredPermissions/type",
															keyword: "type",
															params: { type: "array" },
															message: "must be array"
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
				}
			}
		} else {
			validate61.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate61.errors = vErrors;
		return true;
	}
	validate61.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate135(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate135.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.providers === void 0 && (missing0 = "providers")) {
					validate135.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "providers")) {
						validate135.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.providers !== void 0) {
							let data0 = data.providers;
							if (errors === errors) {
								if (Array.isArray(data0)) {
									const len0 = data0.length;
									for (let i0 = 0; i0 < len0; i0++) {
										const _errs4 = errors;
										if (!validate61(data0[i0], {
											instancePath: instancePath + "/providers/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate135.errors = [{
										instancePath: instancePath + "/providers",
										schemaPath: "#/properties/providers/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
						}
					}
				}
			} else {
				validate135.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate135.errors = vErrors;
		return errors === 0;
	}
	validate135.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderDefinition = validate137;
	function validate137(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate137.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.displayName === void 0 && (missing0 = "displayName") || data.environment === void 0 && (missing0 = "environment") || data.available === void 0 && (missing0 = "available") || data.helpText === void 0 && (missing0 = "helpText") || data.fields === void 0 && (missing0 = "fields") || data.requiredPermissions === void 0 && (missing0 = "requiredPermissions") || data.optionalPermissions === void 0 && (missing0 = "optionalPermissions") || data.forbiddenPermissions === void 0 && (missing0 = "forbiddenPermissions")) {
				validate137.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema87.properties, key0)) {
					validate137.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.available !== void 0) {
					if (typeof data.available !== "boolean") {
						validate137.errors = [{
							instancePath: instancePath + "/available",
							schemaPath: "#/properties/available/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.displayName !== void 0) {
						if (typeof data.displayName !== "string") {
							validate137.errors = [{
								instancePath: instancePath + "/displayName",
								schemaPath: "#/properties/displayName/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.environment !== void 0) {
							if (typeof data.environment !== "string") {
								validate137.errors = [{
									instancePath: instancePath + "/environment",
									schemaPath: "#/properties/environment/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.fields !== void 0) {
								let data3 = data.fields;
								if (Array.isArray(data3)) {
									const len0 = data3.length;
									for (let i0 = 0; i0 < len0; i0++) {
										let data4 = data3[i0];
										if (data4 && typeof data4 == "object" && !Array.isArray(data4)) {
											let missing1;
											if (data4.id === void 0 && (missing1 = "id") || data4.label === void 0 && (missing1 = "label") || data4.inputType === void 0 && (missing1 = "inputType") || data4.required === void 0 && (missing1 = "required") || data4.secret === void 0 && (missing1 = "secret") || data4.maxLength === void 0 && (missing1 = "maxLength") || data4.helpText === void 0 && (missing1 = "helpText") || data4.environment === void 0 && (missing1 = "environment")) {
												validate137.errors = [{
													instancePath: instancePath + "/fields/" + i0,
													schemaPath: "#/$defs/ProviderField/required",
													keyword: "required",
													params: { missingProperty: missing1 },
													message: "must have required property '" + missing1 + "'"
												}];
												return false;
											} else {
												for (const key1 in data4) if (!(key1 === "environment" || key1 === "helpText" || key1 === "id" || key1 === "inputType" || key1 === "label" || key1 === "maxLength" || key1 === "required" || key1 === "secret")) {
													validate137.errors = [{
														instancePath: instancePath + "/fields/" + i0,
														schemaPath: "#/$defs/ProviderField/additionalProperties",
														keyword: "additionalProperties",
														params: { additionalProperty: key1 },
														message: "must NOT have additional properties"
													}];
													return false;
												}
												if (data4.environment !== void 0) {
													if (typeof data4.environment !== "string") {
														validate137.errors = [{
															instancePath: instancePath + "/fields/" + i0 + "/environment",
															schemaPath: "#/$defs/ProviderField/properties/environment/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													var valid3 = true;
												} else var valid3 = true;
												if (valid3) {
													if (data4.helpText !== void 0) {
														if (typeof data4.helpText !== "string") {
															validate137.errors = [{
																instancePath: instancePath + "/fields/" + i0 + "/helpText",
																schemaPath: "#/$defs/ProviderField/properties/helpText/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
														var valid3 = true;
													} else var valid3 = true;
													if (valid3) {
														if (data4.id !== void 0) {
															if (typeof data4.id !== "string") {
																validate137.errors = [{
																	instancePath: instancePath + "/fields/" + i0 + "/id",
																	schemaPath: "#/$defs/ProviderField/properties/id/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																}];
																return false;
															}
															var valid3 = true;
														} else var valid3 = true;
														if (valid3) {
															if (data4.inputType !== void 0) {
																if (typeof data4.inputType !== "string") {
																	validate137.errors = [{
																		instancePath: instancePath + "/fields/" + i0 + "/inputType",
																		schemaPath: "#/$defs/ProviderField/properties/inputType/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																var valid3 = true;
															} else var valid3 = true;
															if (valid3) {
																if (data4.label !== void 0) {
																	if (typeof data4.label !== "string") {
																		validate137.errors = [{
																			instancePath: instancePath + "/fields/" + i0 + "/label",
																			schemaPath: "#/$defs/ProviderField/properties/label/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}];
																		return false;
																	}
																	var valid3 = true;
																} else var valid3 = true;
																if (valid3) {
																	if (data4.maxLength !== void 0) {
																		let data10 = data4.maxLength;
																		if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
																			validate137.errors = [{
																				instancePath: instancePath + "/fields/" + i0 + "/maxLength",
																				schemaPath: "#/$defs/ProviderField/properties/maxLength/type",
																				keyword: "type",
																				params: { type: "integer" },
																				message: "must be integer"
																			}];
																			return false;
																		}
																		if (typeof data10 == "number") {
																			if (data10 < 0 || isNaN(data10)) {
																				validate137.errors = [{
																					instancePath: instancePath + "/fields/" + i0 + "/maxLength",
																					schemaPath: "#/$defs/ProviderField/properties/maxLength/minimum",
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
																		var valid3 = true;
																	} else var valid3 = true;
																	if (valid3) {
																		if (data4.required !== void 0) {
																			if (typeof data4.required !== "boolean") {
																				validate137.errors = [{
																					instancePath: instancePath + "/fields/" + i0 + "/required",
																					schemaPath: "#/$defs/ProviderField/properties/required/type",
																					keyword: "type",
																					params: { type: "boolean" },
																					message: "must be boolean"
																				}];
																				return false;
																			}
																			var valid3 = true;
																		} else var valid3 = true;
																		if (valid3) {
																			if (data4.secret !== void 0) {
																				if (typeof data4.secret !== "boolean") {
																					validate137.errors = [{
																						instancePath: instancePath + "/fields/" + i0 + "/secret",
																						schemaPath: "#/$defs/ProviderField/properties/secret/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					}];
																					return false;
																				}
																				var valid3 = true;
																			} else var valid3 = true;
																		}
																	}
																}
															}
														}
													}
												}
											}
										} else {
											validate137.errors = [{
												instancePath: instancePath + "/fields/" + i0,
												schemaPath: "#/$defs/ProviderField/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											}];
											return false;
										}
									}
								} else {
									validate137.errors = [{
										instancePath: instancePath + "/fields",
										schemaPath: "#/properties/fields/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.forbiddenPermissions !== void 0) {
									let data13 = data.forbiddenPermissions;
									if (Array.isArray(data13)) {
										const len1 = data13.length;
										for (let i1 = 0; i1 < len1; i1++) if (typeof data13[i1] !== "string") {
											validate137.errors = [{
												instancePath: instancePath + "/forbiddenPermissions/" + i1,
												schemaPath: "#/properties/forbiddenPermissions/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate137.errors = [{
											instancePath: instancePath + "/forbiddenPermissions",
											schemaPath: "#/properties/forbiddenPermissions/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.helpText !== void 0) {
										if (typeof data.helpText !== "string") {
											validate137.errors = [{
												instancePath: instancePath + "/helpText",
												schemaPath: "#/properties/helpText/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.optionalPermissions !== void 0) {
											let data16 = data.optionalPermissions;
											if (Array.isArray(data16)) {
												const len2 = data16.length;
												for (let i2 = 0; i2 < len2; i2++) if (typeof data16[i2] !== "string") {
													validate137.errors = [{
														instancePath: instancePath + "/optionalPermissions/" + i2,
														schemaPath: "#/properties/optionalPermissions/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate137.errors = [{
													instancePath: instancePath + "/optionalPermissions",
													schemaPath: "#/properties/optionalPermissions/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.providerId !== void 0) {
												if (typeof data.providerId !== "string") {
													validate137.errors = [{
														instancePath: instancePath + "/providerId",
														schemaPath: "#/properties/providerId/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.requiredPermissions !== void 0) {
													let data19 = data.requiredPermissions;
													if (Array.isArray(data19)) {
														const len3 = data19.length;
														for (let i3 = 0; i3 < len3; i3++) if (typeof data19[i3] !== "string") {
															validate137.errors = [{
																instancePath: instancePath + "/requiredPermissions/" + i3,
																schemaPath: "#/properties/requiredPermissions/items/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
													} else {
														validate137.errors = [{
															instancePath: instancePath + "/requiredPermissions",
															schemaPath: "#/properties/requiredPermissions/type",
															keyword: "type",
															params: { type: "array" },
															message: "must be array"
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
				}
			}
		} else {
			validate137.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate137.errors = vErrors;
		return true;
	}
	validate137.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderField = validate138;
	function validate138(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate138.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.label === void 0 && (missing0 = "label") || data.inputType === void 0 && (missing0 = "inputType") || data.required === void 0 && (missing0 = "required") || data.secret === void 0 && (missing0 = "secret") || data.maxLength === void 0 && (missing0 = "maxLength") || data.helpText === void 0 && (missing0 = "helpText") || data.environment === void 0 && (missing0 = "environment")) {
				validate138.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "environment" || key0 === "helpText" || key0 === "id" || key0 === "inputType" || key0 === "label" || key0 === "maxLength" || key0 === "required" || key0 === "secret")) {
					validate138.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.environment !== void 0) {
					if (typeof data.environment !== "string") {
						validate138.errors = [{
							instancePath: instancePath + "/environment",
							schemaPath: "#/properties/environment/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.helpText !== void 0) {
						if (typeof data.helpText !== "string") {
							validate138.errors = [{
								instancePath: instancePath + "/helpText",
								schemaPath: "#/properties/helpText/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.id !== void 0) {
							if (typeof data.id !== "string") {
								validate138.errors = [{
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
							if (data.inputType !== void 0) {
								if (typeof data.inputType !== "string") {
									validate138.errors = [{
										instancePath: instancePath + "/inputType",
										schemaPath: "#/properties/inputType/type",
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
										validate138.errors = [{
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
								if (valid0) {
									if (data.maxLength !== void 0) {
										let data5 = data.maxLength;
										if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5))) {
											validate138.errors = [{
												instancePath: instancePath + "/maxLength",
												schemaPath: "#/properties/maxLength/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data5 == "number") {
											if (data5 < 0 || isNaN(data5)) {
												validate138.errors = [{
													instancePath: instancePath + "/maxLength",
													schemaPath: "#/properties/maxLength/minimum",
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
										if (data.required !== void 0) {
											if (typeof data.required !== "boolean") {
												validate138.errors = [{
													instancePath: instancePath + "/required",
													schemaPath: "#/properties/required/type",
													keyword: "type",
													params: { type: "boolean" },
													message: "must be boolean"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.secret !== void 0) {
												if (typeof data.secret !== "boolean") {
													validate138.errors = [{
														instancePath: instancePath + "/secret",
														schemaPath: "#/properties/secret/type",
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
				}
			}
		} else {
			validate138.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate138.errors = vErrors;
		return true;
	}
	validate138.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderSelection = validate139;
	function validate139(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate139.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment")) {
				validate139.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "environment" || key0 === "providerId")) {
					validate139.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.environment !== void 0) {
					if (typeof data.environment !== "string") {
						validate139.errors = [{
							instancePath: instancePath + "/environment",
							schemaPath: "#/properties/environment/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.providerId !== void 0) {
						if (typeof data.providerId !== "string") {
							validate139.errors = [{
								instancePath: instancePath + "/providerId",
								schemaPath: "#/properties/providerId/type",
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
			validate139.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate139.errors = vErrors;
		return true;
	}
	validate139.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Remediation = validate140;
	function validate140(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate140.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.label === void 0 && (missing0 = "label")) {
				validate140.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "label")) {
					validate140.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.id !== void 0) {
					let data0 = data.id;
					if (typeof data0 === "string") {
						if (func1(data0) < 1) {
							validate140.errors = [{
								instancePath: instancePath + "/id",
								schemaPath: "#/properties/id/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate140.errors = [{
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
							validate140.errors = [{
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
			validate140.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate140.errors = vErrors;
		return true;
	}
	validate140.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ReplyData = validate141;
	var schema85 = {
		"type": "object",
		"properties": {
			"afterSequence": {
				"type": "integer",
				"format": "uint64",
				"maximum": 9007199254740991,
				"minimum": 0
			},
			"aggregateId": {
				"type": "string",
				"minLength": 1
			},
			"aggregateType": {
				"type": "string",
				"enum": [
					"workspace",
					"account",
					"model-gateway",
					"model",
					"risk"
				]
			},
			"lastSequence": {
				"type": "integer",
				"format": "uint64",
				"maximum": 9007199254740991,
				"minimum": 0
			},
			"replayedCount": {
				"type": "integer",
				"format": "uint64",
				"maximum": 9007199254740991,
				"minimum": 0
			}
		},
		"additionalProperties": false,
		"required": [
			"aggregateType",
			"aggregateId",
			"afterSequence",
			"lastSequence",
			"replayedCount"
		]
	};
	var schema82 = {
		"type": "object",
		"properties": {
			"aggregateId": {
				"type": "string",
				"minLength": 1
			},
			"aggregateType": {
				"type": "string",
				"enum": [
					"workspace",
					"account",
					"model-gateway",
					"model",
					"risk"
				]
			},
			"lastSequence": {
				"type": "integer",
				"format": "uint64",
				"maximum": 9007199254740991,
				"minimum": 0
			},
			"projection": { "$ref": "#/$defs/DomainProjection" }
		},
		"additionalProperties": false,
		"required": [
			"aggregateType",
			"aggregateId",
			"projection",
			"lastSequence"
		]
	};
	function validate52(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate52.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
					validate52.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "projection")) {
						validate52.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.aggregateId !== void 0) {
							let data0 = data.aggregateId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate52.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate52.errors = [{
										instancePath: instancePath + "/aggregateId",
										schemaPath: "#/properties/aggregateId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.aggregateType !== void 0) {
								let data1 = data.aggregateType;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate52.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway" || data1 === "model" || data1 === "risk")) {
									validate52.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema82.properties.aggregateType.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.lastSequence !== void 0) {
									let data2 = data.lastSequence;
									const _errs6 = errors;
									if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
										validate52.errors = [{
											instancePath: instancePath + "/lastSequence",
											schemaPath: "#/properties/lastSequence/type",
											keyword: "type",
											params: { type: "integer" },
											message: "must be integer"
										}];
										return false;
									}
									if (errors === _errs6) {
										if (typeof data2 == "number") {
											if (data2 > 9007199254740991 || isNaN(data2)) {
												validate52.errors = [{
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
												validate52.errors = [{
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
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.projection !== void 0) {
										const _errs8 = errors;
										if (!validate24(data.projection, {
											instancePath: instancePath + "/projection",
											parentData: data,
											parentDataProperty: "projection",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
											errors = vErrors.length;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
								}
							}
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
		}
		validate52.errors = vErrors;
		return errors === 0;
	}
	validate52.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate55(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate55.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate55.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate55.errors = [{
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
									validate55.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate55.errors = [{
											instancePath: instancePath + "/components/" + i0,
											schemaPath: "#/$defs/RuntimeComponent/additionalProperties",
											keyword: "additionalProperties",
											params: { additionalProperty: key1 },
											message: "must NOT have additional properties"
										}];
										return false;
									}
									if (data1.id !== void 0) {
										let data2 = data1.id;
										if (typeof data2 === "string") {
											if (func1(data2) < 1) {
												validate55.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/id",
													schemaPath: "#/$defs/RuntimeComponent/properties/id/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate55.errors = [{
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
												validate55.errors = [{
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
													validate55.errors = [{
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
								validate55.errors = [{
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
						validate55.errors = [{
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
							validate55.errors = [{
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
								validate55.errors = [{
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
	function validate60(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate60.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.providers === void 0 && (missing0 = "providers")) {
					validate60.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "providers")) {
						validate60.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.providers !== void 0) {
							let data0 = data.providers;
							if (errors === errors) {
								if (Array.isArray(data0)) {
									const len0 = data0.length;
									for (let i0 = 0; i0 < len0; i0++) {
										const _errs4 = errors;
										if (!validate61(data0[i0], {
											instancePath: instancePath + "/providers/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate60.errors = [{
										instancePath: instancePath + "/providers",
										schemaPath: "#/properties/providers/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
						}
					}
				}
			} else {
				validate60.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate60.errors = vErrors;
		return errors === 0;
	}
	validate60.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate65(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate65.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.accounts === void 0 && (missing0 = "accounts")) {
					validate65.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "accounts")) {
						validate65.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.accounts !== void 0) {
							let data0 = data.accounts;
							if (errors === errors) {
								if (Array.isArray(data0)) {
									const len0 = data0.length;
									for (let i0 = 0; i0 < len0; i0++) {
										const _errs4 = errors;
										if (!validate39(data0[i0], {
											instancePath: instancePath + "/accounts/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate65.errors = [{
										instancePath: instancePath + "/accounts",
										schemaPath: "#/properties/accounts/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									}];
									return false;
								}
							}
						}
					}
				}
			} else {
				validate65.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate65.errors = vErrors;
		return errors === 0;
	}
	validate65.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate141(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate141.evaluated;
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
												if (errors === _errs15) {
													if (typeof data5 == "number") {
														if (data5 > 5 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 5
																},
																message: "must be <= 5"
															};
															if (vErrors === null) vErrors = [err8];
															else vErrors.push(err8);
															errors++;
														} else if (data5 < 1 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 1
																},
																message: "must be >= 1"
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
													let data6 = data.workspaceId;
													const _errs17 = errors;
													if (errors === _errs17) {
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																const err10 = {
																	instancePath: instancePath + "/workspaceId",
																	schemaPath: "#/$defs/Workspace/properties/workspaceId/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																};
																if (vErrors === null) vErrors = [err10];
																else vErrors.push(err10);
																errors++;
															}
														} else {
															const err11 = {
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
														}
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
				const err12 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs19 = errors;
		if (!validate52(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate52.errors : vErrors.concat(validate52.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
		if (!validate55(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate55.errors : vErrors.concat(validate55.errors);
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
					const err13 = {
						instancePath,
						schemaPath: "#/$defs/SubscriptionAck/required",
						keyword: "required",
						params: { missingProperty: missing1 },
						message: "must have required property '" + missing1 + "'"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				} else {
					const _errs24 = errors;
					for (const key1 in data) if (!(key1 === "afterSequence" || key1 === "aggregateId" || key1 === "aggregateType" || key1 === "lastSequence" || key1 === "replayedCount")) {
						const err14 = {
							instancePath,
							schemaPath: "#/$defs/SubscriptionAck/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
						break;
					}
					if (_errs24 === errors) {
						if (data.afterSequence !== void 0) {
							let data7 = data.afterSequence;
							const _errs25 = errors;
							if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
								const err15 = {
									instancePath: instancePath + "/afterSequence",
									schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (errors === _errs25) {
								if (typeof data7 == "number") {
									if (data7 > 9007199254740991 || isNaN(data7)) {
										const err16 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 9007199254740991
											},
											message: "must be <= 9007199254740991"
										};
										if (vErrors === null) vErrors = [err16];
										else vErrors.push(err16);
										errors++;
									} else if (data7 < 0 || isNaN(data7)) {
										const err17 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err17];
										else vErrors.push(err17);
										errors++;
									}
								}
							}
							var valid4 = _errs25 === errors;
						} else var valid4 = true;
						if (valid4) {
							if (data.aggregateId !== void 0) {
								let data8 = data.aggregateId;
								const _errs27 = errors;
								if (errors === _errs27) {
									if (typeof data8 === "string") {
										if (func1(data8) < 1) {
											const err18 = {
												instancePath: instancePath + "/aggregateId",
												schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/minLength",
												keyword: "minLength",
												params: { limit: 1 },
												message: "must NOT have fewer than 1 characters"
											};
											if (vErrors === null) vErrors = [err18];
											else vErrors.push(err18);
											errors++;
										}
									} else {
										const err19 = {
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err19];
										else vErrors.push(err19);
										errors++;
									}
								}
								var valid4 = _errs27 === errors;
							} else var valid4 = true;
							if (valid4) {
								if (data.aggregateType !== void 0) {
									let data9 = data.aggregateType;
									const _errs29 = errors;
									if (typeof data9 !== "string") {
										const err20 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err20];
										else vErrors.push(err20);
										errors++;
									}
									if (!(data9 === "workspace" || data9 === "account" || data9 === "model-gateway" || data9 === "model" || data9 === "risk")) {
										const err21 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/enum",
											keyword: "enum",
											params: { allowedValues: schema85.properties.aggregateType.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err21];
										else vErrors.push(err21);
										errors++;
									}
									var valid4 = _errs29 === errors;
								} else var valid4 = true;
								if (valid4) {
									if (data.lastSequence !== void 0) {
										let data10 = data.lastSequence;
										const _errs31 = errors;
										if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
											const err22 = {
												instancePath: instancePath + "/lastSequence",
												schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											};
											if (vErrors === null) vErrors = [err22];
											else vErrors.push(err22);
											errors++;
										}
										if (errors === _errs31) {
											if (typeof data10 == "number") {
												if (data10 > 9007199254740991 || isNaN(data10)) {
													const err23 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/maximum",
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
												} else if (data10 < 0 || isNaN(data10)) {
													const err24 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/minimum",
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
										var valid4 = _errs31 === errors;
									} else var valid4 = true;
									if (valid4) {
										if (data.replayedCount !== void 0) {
											let data11 = data.replayedCount;
											const _errs33 = errors;
											if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11))) {
												const err25 = {
													instancePath: instancePath + "/replayedCount",
													schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												};
												if (vErrors === null) vErrors = [err25];
												else vErrors.push(err25);
												errors++;
											}
											if (errors === _errs33) {
												if (typeof data11 == "number") {
													if (data11 > 9007199254740991 || isNaN(data11)) {
														const err26 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 9007199254740991
															},
															message: "must be <= 9007199254740991"
														};
														if (vErrors === null) vErrors = [err26];
														else vErrors.push(err26);
														errors++;
													} else if (data11 < 0 || isNaN(data11)) {
														const err27 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 0
															},
															message: "must be >= 0"
														};
														if (vErrors === null) vErrors = [err27];
														else vErrors.push(err27);
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
				const err28 = {
					instancePath,
					schemaPath: "#/$defs/SubscriptionAck/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err28];
				else vErrors.push(err28);
				errors++;
			}
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs35 = errors;
		if (!validate25(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs35 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs36 = errors;
		if (!validate27(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate27.errors : vErrors.concat(validate27.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs36 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs37 = errors;
		if (!validate43(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs37 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs38 = errors;
		if (!validate60(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate60.errors : vErrors.concat(validate60.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs38 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs39 = errors;
		if (!validate61(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs39 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs40 = errors;
		if (!validate65(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate65.errors : vErrors.concat(validate65.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs40 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs41 = errors;
		if (!validate39(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs41 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs42 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing2;
				if (data.scope === void 0 && (missing2 = "scope") || data.detected === void 0 && (missing2 = "detected") || data.forbidden === void 0 && (missing2 = "forbidden") || data.unsupported === void 0 && (missing2 = "unsupported") || data.acknowledged === void 0 && (missing2 = "acknowledged") || data.ipAllowListStatus === void 0 && (missing2 = "ipAllowListStatus")) {
					const err29 = {
						instancePath,
						schemaPath: "#/$defs/PermissionReview/required",
						keyword: "required",
						params: { missingProperty: missing2 },
						message: "must have required property '" + missing2 + "'"
					};
					if (vErrors === null) vErrors = [err29];
					else vErrors.push(err29);
					errors++;
				} else {
					const _errs45 = errors;
					for (const key2 in data) if (!(key2 === "acknowledged" || key2 === "detected" || key2 === "forbidden" || key2 === "ipAllowList" || key2 === "ipAllowListStatus" || key2 === "scope" || key2 === "unsupported")) {
						const err30 = {
							instancePath,
							schemaPath: "#/$defs/PermissionReview/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key2 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err30];
						else vErrors.push(err30);
						errors++;
						break;
					}
					if (_errs45 === errors) {
						if (data.acknowledged !== void 0) {
							const _errs46 = errors;
							if (typeof data.acknowledged !== "boolean") {
								const err31 = {
									instancePath: instancePath + "/acknowledged",
									schemaPath: "#/$defs/PermissionReview/properties/acknowledged/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								};
								if (vErrors === null) vErrors = [err31];
								else vErrors.push(err31);
								errors++;
							}
							var valid6 = _errs46 === errors;
						} else var valid6 = true;
						if (valid6) {
							if (data.detected !== void 0) {
								let data13 = data.detected;
								const _errs48 = errors;
								if (errors === _errs48) {
									if (Array.isArray(data13)) {
										const len0 = data13.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs50 = errors;
											if (typeof data13[i0] !== "string") {
												const err32 = {
													instancePath: instancePath + "/detected/" + i0,
													schemaPath: "#/$defs/PermissionReview/properties/detected/items/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err32];
												else vErrors.push(err32);
												errors++;
											}
											if (!(_errs50 === errors)) break;
										}
									} else {
										const err33 = {
											instancePath: instancePath + "/detected",
											schemaPath: "#/$defs/PermissionReview/properties/detected/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										};
										if (vErrors === null) vErrors = [err33];
										else vErrors.push(err33);
										errors++;
									}
								}
								var valid6 = _errs48 === errors;
							} else var valid6 = true;
							if (valid6) {
								if (data.forbidden !== void 0) {
									let data15 = data.forbidden;
									const _errs52 = errors;
									if (errors === _errs52) {
										if (Array.isArray(data15)) {
											const len1 = data15.length;
											for (let i1 = 0; i1 < len1; i1++) {
												const _errs54 = errors;
												if (typeof data15[i1] !== "string") {
													const err34 = {
														instancePath: instancePath + "/forbidden/" + i1,
														schemaPath: "#/$defs/PermissionReview/properties/forbidden/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													};
													if (vErrors === null) vErrors = [err34];
													else vErrors.push(err34);
													errors++;
												}
												if (!(_errs54 === errors)) break;
											}
										} else {
											const err35 = {
												instancePath: instancePath + "/forbidden",
												schemaPath: "#/$defs/PermissionReview/properties/forbidden/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											};
											if (vErrors === null) vErrors = [err35];
											else vErrors.push(err35);
											errors++;
										}
									}
									var valid6 = _errs52 === errors;
								} else var valid6 = true;
								if (valid6) {
									if (data.ipAllowList !== void 0) {
										let data17 = data.ipAllowList;
										const _errs56 = errors;
										if (!Array.isArray(data17) && data17 !== null) {
											const err36 = {
												instancePath: instancePath + "/ipAllowList",
												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
												keyword: "type",
												params: { type: schema69.properties.ipAllowList.type },
												message: "must be array,null"
											};
											if (vErrors === null) vErrors = [err36];
											else vErrors.push(err36);
											errors++;
										}
										if (errors === _errs56) {
											if (Array.isArray(data17)) {
												const len2 = data17.length;
												for (let i2 = 0; i2 < len2; i2++) {
													const _errs58 = errors;
													if (typeof data17[i2] !== "string") {
														const err37 = {
															instancePath: instancePath + "/ipAllowList/" + i2,
															schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/items/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														};
														if (vErrors === null) vErrors = [err37];
														else vErrors.push(err37);
														errors++;
													}
													if (!(_errs58 === errors)) break;
												}
											}
										}
										var valid6 = _errs56 === errors;
									} else var valid6 = true;
									if (valid6) {
										if (data.ipAllowListStatus !== void 0) {
											const _errs60 = errors;
											if (typeof data.ipAllowListStatus !== "string") {
												const err38 = {
													instancePath: instancePath + "/ipAllowListStatus",
													schemaPath: "#/$defs/PermissionReview/properties/ipAllowListStatus/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err38];
												else vErrors.push(err38);
												errors++;
											}
											var valid6 = _errs60 === errors;
										} else var valid6 = true;
										if (valid6) {
											if (data.scope !== void 0) {
												let data20 = data.scope;
												const _errs62 = errors;
												if (typeof data20 !== "string") {
													const err39 = {
														instancePath: instancePath + "/scope",
														schemaPath: "#/$defs/PermissionReview/properties/scope/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													};
													if (vErrors === null) vErrors = [err39];
													else vErrors.push(err39);
													errors++;
												}
												if (!(data20 === "VERIFIED" || data20 === "UNVERIFIED")) {
													const err40 = {
														instancePath: instancePath + "/scope",
														schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
														keyword: "enum",
														params: { allowedValues: schema69.properties.scope.enum },
														message: "must be equal to one of the allowed values"
													};
													if (vErrors === null) vErrors = [err40];
													else vErrors.push(err40);
													errors++;
												}
												var valid6 = _errs62 === errors;
											} else var valid6 = true;
											if (valid6) {
												if (data.unsupported !== void 0) {
													let data21 = data.unsupported;
													const _errs64 = errors;
													if (errors === _errs64) {
														if (Array.isArray(data21)) {
															const len3 = data21.length;
															for (let i3 = 0; i3 < len3; i3++) {
																const _errs66 = errors;
																if (typeof data21[i3] !== "string") {
																	const err41 = {
																		instancePath: instancePath + "/unsupported/" + i3,
																		schemaPath: "#/$defs/PermissionReview/properties/unsupported/items/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	};
																	if (vErrors === null) vErrors = [err41];
																	else vErrors.push(err41);
																	errors++;
																}
																if (!(_errs66 === errors)) break;
															}
														} else {
															const err42 = {
																instancePath: instancePath + "/unsupported",
																schemaPath: "#/$defs/PermissionReview/properties/unsupported/type",
																keyword: "type",
																params: { type: "array" },
																message: "must be array"
															};
															if (vErrors === null) vErrors = [err42];
															else vErrors.push(err42);
															errors++;
														}
													}
													var valid6 = _errs64 === errors;
												} else var valid6 = true;
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				const err43 = {
					instancePath,
					schemaPath: "#/$defs/PermissionReview/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err43];
				else vErrors.push(err43);
				errors++;
			}
		}
		var _valid0 = _errs42 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err44 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err44];
			else vErrors.push(err44);
			errors++;
			validate141.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate141.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate141.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.ResultEnvelope = validate151;
	function validate51(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate51.evaluated;
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
												if (errors === _errs15) {
													if (typeof data5 == "number") {
														if (data5 > 5 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 5
																},
																message: "must be <= 5"
															};
															if (vErrors === null) vErrors = [err8];
															else vErrors.push(err8);
															errors++;
														} else if (data5 < 1 || isNaN(data5)) {
															const err9 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 1
																},
																message: "must be >= 1"
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
													let data6 = data.workspaceId;
													const _errs17 = errors;
													if (errors === _errs17) {
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																const err10 = {
																	instancePath: instancePath + "/workspaceId",
																	schemaPath: "#/$defs/Workspace/properties/workspaceId/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																};
																if (vErrors === null) vErrors = [err10];
																else vErrors.push(err10);
																errors++;
															}
														} else {
															const err11 = {
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/$defs/Workspace/properties/workspaceId/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															};
															if (vErrors === null) vErrors = [err11];
															else vErrors.push(err11);
															errors++;
														}
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
				const err12 = {
					instancePath,
					schemaPath: "#/$defs/Workspace/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs19 = errors;
		if (!validate52(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate52.errors : vErrors.concat(validate52.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
		if (!validate55(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate55.errors : vErrors.concat(validate55.errors);
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
					const err13 = {
						instancePath,
						schemaPath: "#/$defs/SubscriptionAck/required",
						keyword: "required",
						params: { missingProperty: missing1 },
						message: "must have required property '" + missing1 + "'"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				} else {
					const _errs24 = errors;
					for (const key1 in data) if (!(key1 === "afterSequence" || key1 === "aggregateId" || key1 === "aggregateType" || key1 === "lastSequence" || key1 === "replayedCount")) {
						const err14 = {
							instancePath,
							schemaPath: "#/$defs/SubscriptionAck/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
						break;
					}
					if (_errs24 === errors) {
						if (data.afterSequence !== void 0) {
							let data7 = data.afterSequence;
							const _errs25 = errors;
							if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
								const err15 = {
									instancePath: instancePath + "/afterSequence",
									schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (errors === _errs25) {
								if (typeof data7 == "number") {
									if (data7 > 9007199254740991 || isNaN(data7)) {
										const err16 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 9007199254740991
											},
											message: "must be <= 9007199254740991"
										};
										if (vErrors === null) vErrors = [err16];
										else vErrors.push(err16);
										errors++;
									} else if (data7 < 0 || isNaN(data7)) {
										const err17 = {
											instancePath: instancePath + "/afterSequence",
											schemaPath: "#/$defs/SubscriptionAck/properties/afterSequence/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err17];
										else vErrors.push(err17);
										errors++;
									}
								}
							}
							var valid4 = _errs25 === errors;
						} else var valid4 = true;
						if (valid4) {
							if (data.aggregateId !== void 0) {
								let data8 = data.aggregateId;
								const _errs27 = errors;
								if (errors === _errs27) {
									if (typeof data8 === "string") {
										if (func1(data8) < 1) {
											const err18 = {
												instancePath: instancePath + "/aggregateId",
												schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/minLength",
												keyword: "minLength",
												params: { limit: 1 },
												message: "must NOT have fewer than 1 characters"
											};
											if (vErrors === null) vErrors = [err18];
											else vErrors.push(err18);
											errors++;
										}
									} else {
										const err19 = {
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err19];
										else vErrors.push(err19);
										errors++;
									}
								}
								var valid4 = _errs27 === errors;
							} else var valid4 = true;
							if (valid4) {
								if (data.aggregateType !== void 0) {
									let data9 = data.aggregateType;
									const _errs29 = errors;
									if (typeof data9 !== "string") {
										const err20 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										if (vErrors === null) vErrors = [err20];
										else vErrors.push(err20);
										errors++;
									}
									if (!(data9 === "workspace" || data9 === "account" || data9 === "model-gateway" || data9 === "model" || data9 === "risk")) {
										const err21 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/enum",
											keyword: "enum",
											params: { allowedValues: schema85.properties.aggregateType.enum },
											message: "must be equal to one of the allowed values"
										};
										if (vErrors === null) vErrors = [err21];
										else vErrors.push(err21);
										errors++;
									}
									var valid4 = _errs29 === errors;
								} else var valid4 = true;
								if (valid4) {
									if (data.lastSequence !== void 0) {
										let data10 = data.lastSequence;
										const _errs31 = errors;
										if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10))) {
											const err22 = {
												instancePath: instancePath + "/lastSequence",
												schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											};
											if (vErrors === null) vErrors = [err22];
											else vErrors.push(err22);
											errors++;
										}
										if (errors === _errs31) {
											if (typeof data10 == "number") {
												if (data10 > 9007199254740991 || isNaN(data10)) {
													const err23 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/maximum",
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
												} else if (data10 < 0 || isNaN(data10)) {
													const err24 = {
														instancePath: instancePath + "/lastSequence",
														schemaPath: "#/$defs/SubscriptionAck/properties/lastSequence/minimum",
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
										var valid4 = _errs31 === errors;
									} else var valid4 = true;
									if (valid4) {
										if (data.replayedCount !== void 0) {
											let data11 = data.replayedCount;
											const _errs33 = errors;
											if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11))) {
												const err25 = {
													instancePath: instancePath + "/replayedCount",
													schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												};
												if (vErrors === null) vErrors = [err25];
												else vErrors.push(err25);
												errors++;
											}
											if (errors === _errs33) {
												if (typeof data11 == "number") {
													if (data11 > 9007199254740991 || isNaN(data11)) {
														const err26 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 9007199254740991
															},
															message: "must be <= 9007199254740991"
														};
														if (vErrors === null) vErrors = [err26];
														else vErrors.push(err26);
														errors++;
													} else if (data11 < 0 || isNaN(data11)) {
														const err27 = {
															instancePath: instancePath + "/replayedCount",
															schemaPath: "#/$defs/SubscriptionAck/properties/replayedCount/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 0
															},
															message: "must be >= 0"
														};
														if (vErrors === null) vErrors = [err27];
														else vErrors.push(err27);
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
				const err28 = {
					instancePath,
					schemaPath: "#/$defs/SubscriptionAck/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err28];
				else vErrors.push(err28);
				errors++;
			}
		}
		var _valid0 = _errs21 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs35 = errors;
		if (!validate25(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs35 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs36 = errors;
		if (!validate27(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate27.errors : vErrors.concat(validate27.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs36 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs37 = errors;
		if (!validate43(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs37 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs38 = errors;
		if (!validate60(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate60.errors : vErrors.concat(validate60.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs38 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs39 = errors;
		if (!validate61(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs39 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs40 = errors;
		if (!validate65(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate65.errors : vErrors.concat(validate65.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs40 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs41 = errors;
		if (!validate39(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs41 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs42 = errors;
		if (errors === errors) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing2;
				if (data.scope === void 0 && (missing2 = "scope") || data.detected === void 0 && (missing2 = "detected") || data.forbidden === void 0 && (missing2 = "forbidden") || data.unsupported === void 0 && (missing2 = "unsupported") || data.acknowledged === void 0 && (missing2 = "acknowledged") || data.ipAllowListStatus === void 0 && (missing2 = "ipAllowListStatus")) {
					const err29 = {
						instancePath,
						schemaPath: "#/$defs/PermissionReview/required",
						keyword: "required",
						params: { missingProperty: missing2 },
						message: "must have required property '" + missing2 + "'"
					};
					if (vErrors === null) vErrors = [err29];
					else vErrors.push(err29);
					errors++;
				} else {
					const _errs45 = errors;
					for (const key2 in data) if (!(key2 === "acknowledged" || key2 === "detected" || key2 === "forbidden" || key2 === "ipAllowList" || key2 === "ipAllowListStatus" || key2 === "scope" || key2 === "unsupported")) {
						const err30 = {
							instancePath,
							schemaPath: "#/$defs/PermissionReview/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key2 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err30];
						else vErrors.push(err30);
						errors++;
						break;
					}
					if (_errs45 === errors) {
						if (data.acknowledged !== void 0) {
							const _errs46 = errors;
							if (typeof data.acknowledged !== "boolean") {
								const err31 = {
									instancePath: instancePath + "/acknowledged",
									schemaPath: "#/$defs/PermissionReview/properties/acknowledged/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								};
								if (vErrors === null) vErrors = [err31];
								else vErrors.push(err31);
								errors++;
							}
							var valid6 = _errs46 === errors;
						} else var valid6 = true;
						if (valid6) {
							if (data.detected !== void 0) {
								let data13 = data.detected;
								const _errs48 = errors;
								if (errors === _errs48) {
									if (Array.isArray(data13)) {
										const len0 = data13.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs50 = errors;
											if (typeof data13[i0] !== "string") {
												const err32 = {
													instancePath: instancePath + "/detected/" + i0,
													schemaPath: "#/$defs/PermissionReview/properties/detected/items/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err32];
												else vErrors.push(err32);
												errors++;
											}
											if (!(_errs50 === errors)) break;
										}
									} else {
										const err33 = {
											instancePath: instancePath + "/detected",
											schemaPath: "#/$defs/PermissionReview/properties/detected/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										};
										if (vErrors === null) vErrors = [err33];
										else vErrors.push(err33);
										errors++;
									}
								}
								var valid6 = _errs48 === errors;
							} else var valid6 = true;
							if (valid6) {
								if (data.forbidden !== void 0) {
									let data15 = data.forbidden;
									const _errs52 = errors;
									if (errors === _errs52) {
										if (Array.isArray(data15)) {
											const len1 = data15.length;
											for (let i1 = 0; i1 < len1; i1++) {
												const _errs54 = errors;
												if (typeof data15[i1] !== "string") {
													const err34 = {
														instancePath: instancePath + "/forbidden/" + i1,
														schemaPath: "#/$defs/PermissionReview/properties/forbidden/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													};
													if (vErrors === null) vErrors = [err34];
													else vErrors.push(err34);
													errors++;
												}
												if (!(_errs54 === errors)) break;
											}
										} else {
											const err35 = {
												instancePath: instancePath + "/forbidden",
												schemaPath: "#/$defs/PermissionReview/properties/forbidden/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											};
											if (vErrors === null) vErrors = [err35];
											else vErrors.push(err35);
											errors++;
										}
									}
									var valid6 = _errs52 === errors;
								} else var valid6 = true;
								if (valid6) {
									if (data.ipAllowList !== void 0) {
										let data17 = data.ipAllowList;
										const _errs56 = errors;
										if (!Array.isArray(data17) && data17 !== null) {
											const err36 = {
												instancePath: instancePath + "/ipAllowList",
												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
												keyword: "type",
												params: { type: schema69.properties.ipAllowList.type },
												message: "must be array,null"
											};
											if (vErrors === null) vErrors = [err36];
											else vErrors.push(err36);
											errors++;
										}
										if (errors === _errs56) {
											if (Array.isArray(data17)) {
												const len2 = data17.length;
												for (let i2 = 0; i2 < len2; i2++) {
													const _errs58 = errors;
													if (typeof data17[i2] !== "string") {
														const err37 = {
															instancePath: instancePath + "/ipAllowList/" + i2,
															schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/items/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														};
														if (vErrors === null) vErrors = [err37];
														else vErrors.push(err37);
														errors++;
													}
													if (!(_errs58 === errors)) break;
												}
											}
										}
										var valid6 = _errs56 === errors;
									} else var valid6 = true;
									if (valid6) {
										if (data.ipAllowListStatus !== void 0) {
											const _errs60 = errors;
											if (typeof data.ipAllowListStatus !== "string") {
												const err38 = {
													instancePath: instancePath + "/ipAllowListStatus",
													schemaPath: "#/$defs/PermissionReview/properties/ipAllowListStatus/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												if (vErrors === null) vErrors = [err38];
												else vErrors.push(err38);
												errors++;
											}
											var valid6 = _errs60 === errors;
										} else var valid6 = true;
										if (valid6) {
											if (data.scope !== void 0) {
												let data20 = data.scope;
												const _errs62 = errors;
												if (typeof data20 !== "string") {
													const err39 = {
														instancePath: instancePath + "/scope",
														schemaPath: "#/$defs/PermissionReview/properties/scope/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													};
													if (vErrors === null) vErrors = [err39];
													else vErrors.push(err39);
													errors++;
												}
												if (!(data20 === "VERIFIED" || data20 === "UNVERIFIED")) {
													const err40 = {
														instancePath: instancePath + "/scope",
														schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
														keyword: "enum",
														params: { allowedValues: schema69.properties.scope.enum },
														message: "must be equal to one of the allowed values"
													};
													if (vErrors === null) vErrors = [err40];
													else vErrors.push(err40);
													errors++;
												}
												var valid6 = _errs62 === errors;
											} else var valid6 = true;
											if (valid6) {
												if (data.unsupported !== void 0) {
													let data21 = data.unsupported;
													const _errs64 = errors;
													if (errors === _errs64) {
														if (Array.isArray(data21)) {
															const len3 = data21.length;
															for (let i3 = 0; i3 < len3; i3++) {
																const _errs66 = errors;
																if (typeof data21[i3] !== "string") {
																	const err41 = {
																		instancePath: instancePath + "/unsupported/" + i3,
																		schemaPath: "#/$defs/PermissionReview/properties/unsupported/items/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	};
																	if (vErrors === null) vErrors = [err41];
																	else vErrors.push(err41);
																	errors++;
																}
																if (!(_errs66 === errors)) break;
															}
														} else {
															const err42 = {
																instancePath: instancePath + "/unsupported",
																schemaPath: "#/$defs/PermissionReview/properties/unsupported/type",
																keyword: "type",
																params: { type: "array" },
																message: "must be array"
															};
															if (vErrors === null) vErrors = [err42];
															else vErrors.push(err42);
															errors++;
														}
													}
													var valid6 = _errs64 === errors;
												} else var valid6 = true;
											}
										}
									}
								}
							}
						}
					}
				}
			} else {
				const err43 = {
					instancePath,
					schemaPath: "#/$defs/PermissionReview/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err43];
				else vErrors.push(err43);
				errors++;
			}
		}
		var _valid0 = _errs42 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err44 = {
				instancePath,
				schemaPath: "#/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			if (vErrors === null) vErrors = [err44];
			else vErrors.push(err44);
			errors++;
			validate51.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate51.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate51.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate50(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate50.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate50.errors = [{
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
						validate50.errors = [{
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
							if (!validate51(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate50.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate50.errors = [{
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
									let data2 = data.requestId;
									const _errs5 = errors;
									if (errors === _errs5) {
										if (typeof data2 === "string") {
											if (func1(data2) > 128) {
												validate50.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate50.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate50.errors = [{
												instancePath: instancePath + "/requestId",
												schemaPath: "#/properties/requestId/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate50.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate50.errors = [{
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
													validate50.errors = [{
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
											if (errors === _errs9) {
												if (typeof data4 === "string") {
													if (func1(data4) < 1) {
														validate50.errors = [{
															instancePath: instancePath + "/stateVersion",
															schemaPath: "#/properties/stateVersion/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate50.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
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
				validate50.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate50.errors = vErrors;
		return errors === 0;
	}
	validate50.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate71(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate71.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate71.errors = [{
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
						validate71.errors = [{
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
							if (!validate72(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate72.errors : vErrors.concat(validate72.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate71.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate71.errors = [{
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
									let data2 = data.requestId;
									const _errs5 = errors;
									if (errors === _errs5) {
										if (typeof data2 === "string") {
											if (func1(data2) > 128) {
												validate71.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate71.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate71.errors = [{
												instancePath: instancePath + "/requestId",
												schemaPath: "#/properties/requestId/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate71.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate71.errors = [{
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
													validate71.errors = [{
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
				validate71.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate71.errors = vErrors;
		return errors === 0;
	}
	validate71.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate151(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate151.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate50(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate50.errors : vErrors.concat(validate50.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
		if (!validate71(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate71.errors : vErrors.concat(validate71.errors);
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
			validate151.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate151.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate151.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.RiskPolicy = validate154;
	function validate154(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate154.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const key0 in data) if (!(key0 === "liveInactivityTimeoutMinutes" || key0 === "marketOrdersEnabled" || key0 === "maxDailyRealizedLoss" || key0 === "maxDailyTradedNotional" || key0 === "maxOrderNotional" || key0 === "maxSingleInstrumentExposurePercent" || key0 === "staleQuoteThresholdSeconds")) {
				validate154.errors = [{
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				}];
				return false;
			}
			if (data.liveInactivityTimeoutMinutes !== void 0) {
				let data0 = data.liveInactivityTimeoutMinutes;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0))) {
					validate154.errors = [{
						instancePath: instancePath + "/liveInactivityTimeoutMinutes",
						schemaPath: "#/properties/liveInactivityTimeoutMinutes/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					}];
					return false;
				}
				if (typeof data0 == "number") {
					if (data0 > 1440 || isNaN(data0)) {
						validate154.errors = [{
							instancePath: instancePath + "/liveInactivityTimeoutMinutes",
							schemaPath: "#/properties/liveInactivityTimeoutMinutes/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 1440
							},
							message: "must be <= 1440"
						}];
						return false;
					} else if (data0 < 1 || isNaN(data0)) {
						validate154.errors = [{
							instancePath: instancePath + "/liveInactivityTimeoutMinutes",
							schemaPath: "#/properties/liveInactivityTimeoutMinutes/minimum",
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
			if (valid0) {
				if (data.marketOrdersEnabled !== void 0) {
					if (typeof data.marketOrdersEnabled !== "boolean") {
						validate154.errors = [{
							instancePath: instancePath + "/marketOrdersEnabled",
							schemaPath: "#/properties/marketOrdersEnabled/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.maxDailyRealizedLoss !== void 0) {
						let data2 = data.maxDailyRealizedLoss;
						if (typeof data2 !== "string" && data2 !== null) {
							validate154.errors = [{
								instancePath: instancePath + "/maxDailyRealizedLoss",
								schemaPath: "#/properties/maxDailyRealizedLoss/type",
								keyword: "type",
								params: { type: schema72.properties.maxDailyRealizedLoss.type },
								message: "must be string,null"
							}];
							return false;
						}
						if (typeof data2 === "string") {
							if (func1(data2) > 32) {
								validate154.errors = [{
									instancePath: instancePath + "/maxDailyRealizedLoss",
									schemaPath: "#/properties/maxDailyRealizedLoss/maxLength",
									keyword: "maxLength",
									params: { limit: 32 },
									message: "must NOT have more than 32 characters"
								}];
								return false;
							}
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.maxDailyTradedNotional !== void 0) {
							let data3 = data.maxDailyTradedNotional;
							if (typeof data3 !== "string" && data3 !== null) {
								validate154.errors = [{
									instancePath: instancePath + "/maxDailyTradedNotional",
									schemaPath: "#/properties/maxDailyTradedNotional/type",
									keyword: "type",
									params: { type: schema72.properties.maxDailyTradedNotional.type },
									message: "must be string,null"
								}];
								return false;
							}
							if (typeof data3 === "string") {
								if (func1(data3) > 32) {
									validate154.errors = [{
										instancePath: instancePath + "/maxDailyTradedNotional",
										schemaPath: "#/properties/maxDailyTradedNotional/maxLength",
										keyword: "maxLength",
										params: { limit: 32 },
										message: "must NOT have more than 32 characters"
									}];
									return false;
								}
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.maxOrderNotional !== void 0) {
								let data4 = data.maxOrderNotional;
								if (typeof data4 !== "string" && data4 !== null) {
									validate154.errors = [{
										instancePath: instancePath + "/maxOrderNotional",
										schemaPath: "#/properties/maxOrderNotional/type",
										keyword: "type",
										params: { type: schema72.properties.maxOrderNotional.type },
										message: "must be string,null"
									}];
									return false;
								}
								if (typeof data4 === "string") {
									if (func1(data4) > 32) {
										validate154.errors = [{
											instancePath: instancePath + "/maxOrderNotional",
											schemaPath: "#/properties/maxOrderNotional/maxLength",
											keyword: "maxLength",
											params: { limit: 32 },
											message: "must NOT have more than 32 characters"
										}];
										return false;
									}
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.maxSingleInstrumentExposurePercent !== void 0) {
									let data5 = data.maxSingleInstrumentExposurePercent;
									if (typeof data5 !== "string" && data5 !== null) {
										validate154.errors = [{
											instancePath: instancePath + "/maxSingleInstrumentExposurePercent",
											schemaPath: "#/properties/maxSingleInstrumentExposurePercent/type",
											keyword: "type",
											params: { type: schema72.properties.maxSingleInstrumentExposurePercent.type },
											message: "must be string,null"
										}];
										return false;
									}
									if (typeof data5 === "string") {
										if (func1(data5) > 32) {
											validate154.errors = [{
												instancePath: instancePath + "/maxSingleInstrumentExposurePercent",
												schemaPath: "#/properties/maxSingleInstrumentExposurePercent/maxLength",
												keyword: "maxLength",
												params: { limit: 32 },
												message: "must NOT have more than 32 characters"
											}];
											return false;
										}
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.staleQuoteThresholdSeconds !== void 0) {
										let data6 = data.staleQuoteThresholdSeconds;
										if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
											validate154.errors = [{
												instancePath: instancePath + "/staleQuoteThresholdSeconds",
												schemaPath: "#/properties/staleQuoteThresholdSeconds/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data6 == "number") {
											if (data6 > 86400 || isNaN(data6)) {
												validate154.errors = [{
													instancePath: instancePath + "/staleQuoteThresholdSeconds",
													schemaPath: "#/properties/staleQuoteThresholdSeconds/maximum",
													keyword: "maximum",
													params: {
														comparison: "<=",
														limit: 86400
													},
													message: "must be <= 86400"
												}];
												return false;
											} else if (data6 < 1 || isNaN(data6)) {
												validate154.errors = [{
													instancePath: instancePath + "/staleQuoteThresholdSeconds",
													schemaPath: "#/properties/staleQuoteThresholdSeconds/minimum",
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
		} else {
			validate154.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate154.errors = vErrors;
		return true;
	}
	validate154.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RiskPolicyInput = validate155;
	var schema96 = {
		"type": "object",
		"properties": {
			"liveInactivityTimeoutMinutes": {
				"type": "integer",
				"format": "uint64",
				"maximum": 1440,
				"minimum": 1
			},
			"marketOrdersEnabled": { "type": "boolean" },
			"maxDailyRealizedLoss": {
				"type": ["string", "null"],
				"maxLength": 32
			},
			"maxDailyTradedNotional": {
				"type": ["string", "null"],
				"maxLength": 32
			},
			"maxOrderNotional": {
				"type": ["string", "null"],
				"maxLength": 32
			},
			"maxSingleInstrumentExposurePercent": {
				"type": ["string", "null"],
				"maxLength": 32
			},
			"staleQuoteThresholdSeconds": {
				"type": "integer",
				"format": "uint64",
				"maximum": 86400,
				"minimum": 1
			}
		},
		"additionalProperties": false,
		"required": [
			"maxOrderNotional",
			"maxSingleInstrumentExposurePercent",
			"maxDailyTradedNotional",
			"maxDailyRealizedLoss",
			"staleQuoteThresholdSeconds",
			"marketOrdersEnabled",
			"liveInactivityTimeoutMinutes"
		]
	};
	function validate155(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate155.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.maxOrderNotional === void 0 && (missing0 = "maxOrderNotional") || data.maxSingleInstrumentExposurePercent === void 0 && (missing0 = "maxSingleInstrumentExposurePercent") || data.maxDailyTradedNotional === void 0 && (missing0 = "maxDailyTradedNotional") || data.maxDailyRealizedLoss === void 0 && (missing0 = "maxDailyRealizedLoss") || data.staleQuoteThresholdSeconds === void 0 && (missing0 = "staleQuoteThresholdSeconds") || data.marketOrdersEnabled === void 0 && (missing0 = "marketOrdersEnabled") || data.liveInactivityTimeoutMinutes === void 0 && (missing0 = "liveInactivityTimeoutMinutes")) {
				validate155.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "liveInactivityTimeoutMinutes" || key0 === "marketOrdersEnabled" || key0 === "maxDailyRealizedLoss" || key0 === "maxDailyTradedNotional" || key0 === "maxOrderNotional" || key0 === "maxSingleInstrumentExposurePercent" || key0 === "staleQuoteThresholdSeconds")) {
					validate155.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.liveInactivityTimeoutMinutes !== void 0) {
					let data0 = data.liveInactivityTimeoutMinutes;
					if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0))) {
						validate155.errors = [{
							instancePath: instancePath + "/liveInactivityTimeoutMinutes",
							schemaPath: "#/properties/liveInactivityTimeoutMinutes/type",
							keyword: "type",
							params: { type: "integer" },
							message: "must be integer"
						}];
						return false;
					}
					if (typeof data0 == "number") {
						if (data0 > 1440 || isNaN(data0)) {
							validate155.errors = [{
								instancePath: instancePath + "/liveInactivityTimeoutMinutes",
								schemaPath: "#/properties/liveInactivityTimeoutMinutes/maximum",
								keyword: "maximum",
								params: {
									comparison: "<=",
									limit: 1440
								},
								message: "must be <= 1440"
							}];
							return false;
						} else if (data0 < 1 || isNaN(data0)) {
							validate155.errors = [{
								instancePath: instancePath + "/liveInactivityTimeoutMinutes",
								schemaPath: "#/properties/liveInactivityTimeoutMinutes/minimum",
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
				if (valid0) {
					if (data.marketOrdersEnabled !== void 0) {
						if (typeof data.marketOrdersEnabled !== "boolean") {
							validate155.errors = [{
								instancePath: instancePath + "/marketOrdersEnabled",
								schemaPath: "#/properties/marketOrdersEnabled/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.maxDailyRealizedLoss !== void 0) {
							let data2 = data.maxDailyRealizedLoss;
							if (typeof data2 !== "string" && data2 !== null) {
								validate155.errors = [{
									instancePath: instancePath + "/maxDailyRealizedLoss",
									schemaPath: "#/properties/maxDailyRealizedLoss/type",
									keyword: "type",
									params: { type: schema96.properties.maxDailyRealizedLoss.type },
									message: "must be string,null"
								}];
								return false;
							}
							if (typeof data2 === "string") {
								if (func1(data2) > 32) {
									validate155.errors = [{
										instancePath: instancePath + "/maxDailyRealizedLoss",
										schemaPath: "#/properties/maxDailyRealizedLoss/maxLength",
										keyword: "maxLength",
										params: { limit: 32 },
										message: "must NOT have more than 32 characters"
									}];
									return false;
								}
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.maxDailyTradedNotional !== void 0) {
								let data3 = data.maxDailyTradedNotional;
								if (typeof data3 !== "string" && data3 !== null) {
									validate155.errors = [{
										instancePath: instancePath + "/maxDailyTradedNotional",
										schemaPath: "#/properties/maxDailyTradedNotional/type",
										keyword: "type",
										params: { type: schema96.properties.maxDailyTradedNotional.type },
										message: "must be string,null"
									}];
									return false;
								}
								if (typeof data3 === "string") {
									if (func1(data3) > 32) {
										validate155.errors = [{
											instancePath: instancePath + "/maxDailyTradedNotional",
											schemaPath: "#/properties/maxDailyTradedNotional/maxLength",
											keyword: "maxLength",
											params: { limit: 32 },
											message: "must NOT have more than 32 characters"
										}];
										return false;
									}
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.maxOrderNotional !== void 0) {
									let data4 = data.maxOrderNotional;
									if (typeof data4 !== "string" && data4 !== null) {
										validate155.errors = [{
											instancePath: instancePath + "/maxOrderNotional",
											schemaPath: "#/properties/maxOrderNotional/type",
											keyword: "type",
											params: { type: schema96.properties.maxOrderNotional.type },
											message: "must be string,null"
										}];
										return false;
									}
									if (typeof data4 === "string") {
										if (func1(data4) > 32) {
											validate155.errors = [{
												instancePath: instancePath + "/maxOrderNotional",
												schemaPath: "#/properties/maxOrderNotional/maxLength",
												keyword: "maxLength",
												params: { limit: 32 },
												message: "must NOT have more than 32 characters"
											}];
											return false;
										}
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.maxSingleInstrumentExposurePercent !== void 0) {
										let data5 = data.maxSingleInstrumentExposurePercent;
										if (typeof data5 !== "string" && data5 !== null) {
											validate155.errors = [{
												instancePath: instancePath + "/maxSingleInstrumentExposurePercent",
												schemaPath: "#/properties/maxSingleInstrumentExposurePercent/type",
												keyword: "type",
												params: { type: schema96.properties.maxSingleInstrumentExposurePercent.type },
												message: "must be string,null"
											}];
											return false;
										}
										if (typeof data5 === "string") {
											if (func1(data5) > 32) {
												validate155.errors = [{
													instancePath: instancePath + "/maxSingleInstrumentExposurePercent",
													schemaPath: "#/properties/maxSingleInstrumentExposurePercent/maxLength",
													keyword: "maxLength",
													params: { limit: 32 },
													message: "must NOT have more than 32 characters"
												}];
												return false;
											}
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.staleQuoteThresholdSeconds !== void 0) {
											let data6 = data.staleQuoteThresholdSeconds;
											if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
												validate155.errors = [{
													instancePath: instancePath + "/staleQuoteThresholdSeconds",
													schemaPath: "#/properties/staleQuoteThresholdSeconds/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												}];
												return false;
											}
											if (typeof data6 == "number") {
												if (data6 > 86400 || isNaN(data6)) {
													validate155.errors = [{
														instancePath: instancePath + "/staleQuoteThresholdSeconds",
														schemaPath: "#/properties/staleQuoteThresholdSeconds/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 86400
														},
														message: "must be <= 86400"
													}];
													return false;
												} else if (data6 < 1 || isNaN(data6)) {
													validate155.errors = [{
														instancePath: instancePath + "/staleQuoteThresholdSeconds",
														schemaPath: "#/properties/staleQuoteThresholdSeconds/minimum",
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
		} else {
			validate155.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate155.errors = vErrors;
		return true;
	}
	validate155.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RiskPolicyState = validate156;
	function validate156(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate156.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.policyVersion === void 0 && (missing0 = "policyVersion") || data.configured === void 0 && (missing0 = "configured") || data.onboardingStep === void 0 && (missing0 = "onboardingStep") || data.onboardingCompleted === void 0 && (missing0 = "onboardingCompleted") || data.policy === void 0 && (missing0 = "policy") || data.hardRules === void 0 && (missing0 = "hardRules") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate156.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func31.call(schema70.properties, key0)) {
					validate156.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.configured !== void 0) {
					if (typeof data.configured !== "boolean") {
						validate156.errors = [{
							instancePath: instancePath + "/configured",
							schemaPath: "#/properties/configured/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.hardRules !== void 0) {
						let data1 = data.hardRules;
						if (Array.isArray(data1)) {
							const len0 = data1.length;
							for (let i0 = 0; i0 < len0; i0++) {
								let data2 = data1[i0];
								if (data2 && typeof data2 == "object" && !Array.isArray(data2)) {
									let missing1;
									if (data2.id === void 0 && (missing1 = "id") || data2.description === void 0 && (missing1 = "description")) {
										validate156.errors = [{
											instancePath: instancePath + "/hardRules/" + i0,
											schemaPath: "#/$defs/HardSafetyRule/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "description" || key1 === "id")) {
											validate156.errors = [{
												instancePath: instancePath + "/hardRules/" + i0,
												schemaPath: "#/$defs/HardSafetyRule/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key1 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data2.description !== void 0) {
											let data3 = data2.description;
											if (typeof data3 === "string") {
												if (func1(data3) > 256) {
													validate156.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/description",
														schemaPath: "#/$defs/HardSafetyRule/properties/description/maxLength",
														keyword: "maxLength",
														params: { limit: 256 },
														message: "must NOT have more than 256 characters"
													}];
													return false;
												} else if (func1(data3) < 1) {
													validate156.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/description",
														schemaPath: "#/$defs/HardSafetyRule/properties/description/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate156.errors = [{
													instancePath: instancePath + "/hardRules/" + i0 + "/description",
													schemaPath: "#/$defs/HardSafetyRule/properties/description/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid3 = true;
										} else var valid3 = true;
										if (valid3) {
											if (data2.id !== void 0) {
												let data4 = data2.id;
												if (typeof data4 === "string") {
													if (func1(data4) > 64) {
														validate156.errors = [{
															instancePath: instancePath + "/hardRules/" + i0 + "/id",
															schemaPath: "#/$defs/HardSafetyRule/properties/id/maxLength",
															keyword: "maxLength",
															params: { limit: 64 },
															message: "must NOT have more than 64 characters"
														}];
														return false;
													} else if (func1(data4) < 1) {
														validate156.errors = [{
															instancePath: instancePath + "/hardRules/" + i0 + "/id",
															schemaPath: "#/$defs/HardSafetyRule/properties/id/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate156.errors = [{
														instancePath: instancePath + "/hardRules/" + i0 + "/id",
														schemaPath: "#/$defs/HardSafetyRule/properties/id/type",
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
									validate156.errors = [{
										instancePath: instancePath + "/hardRules/" + i0,
										schemaPath: "#/$defs/HardSafetyRule/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}];
									return false;
								}
							}
						} else {
							validate156.errors = [{
								instancePath: instancePath + "/hardRules",
								schemaPath: "#/properties/hardRules/type",
								keyword: "type",
								params: { type: "array" },
								message: "must be array"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.onboardingCompleted !== void 0) {
							if (typeof data.onboardingCompleted !== "boolean") {
								validate156.errors = [{
									instancePath: instancePath + "/onboardingCompleted",
									schemaPath: "#/properties/onboardingCompleted/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.onboardingStep !== void 0) {
								let data6 = data.onboardingStep;
								if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
									validate156.errors = [{
										instancePath: instancePath + "/onboardingStep",
										schemaPath: "#/properties/onboardingStep/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									}];
									return false;
								}
								if (typeof data6 == "number") {
									if (data6 > 5 || isNaN(data6)) {
										validate156.errors = [{
											instancePath: instancePath + "/onboardingStep",
											schemaPath: "#/properties/onboardingStep/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 5
											},
											message: "must be <= 5"
										}];
										return false;
									} else if (data6 < 1 || isNaN(data6)) {
										validate156.errors = [{
											instancePath: instancePath + "/onboardingStep",
											schemaPath: "#/properties/onboardingStep/minimum",
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
							if (valid0) {
								if (data.policy !== void 0) {
									let data7 = data.policy;
									if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
										for (const key2 in data7) if (!(key2 === "liveInactivityTimeoutMinutes" || key2 === "marketOrdersEnabled" || key2 === "maxDailyRealizedLoss" || key2 === "maxDailyTradedNotional" || key2 === "maxOrderNotional" || key2 === "maxSingleInstrumentExposurePercent" || key2 === "staleQuoteThresholdSeconds")) {
											validate156.errors = [{
												instancePath: instancePath + "/policy",
												schemaPath: "#/$defs/RiskPolicy/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key2 },
												message: "must NOT have additional properties"
											}];
											return false;
										}
										if (data7.liveInactivityTimeoutMinutes !== void 0) {
											let data8 = data7.liveInactivityTimeoutMinutes;
											if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8))) {
												validate156.errors = [{
													instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
													schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												}];
												return false;
											}
											if (typeof data8 == "number") {
												if (data8 > 1440 || isNaN(data8)) {
													validate156.errors = [{
														instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
														schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 1440
														},
														message: "must be <= 1440"
													}];
													return false;
												} else if (data8 < 1 || isNaN(data8)) {
													validate156.errors = [{
														instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
														schemaPath: "#/$defs/RiskPolicy/properties/liveInactivityTimeoutMinutes/minimum",
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
											var valid5 = true;
										} else var valid5 = true;
										if (valid5) {
											if (data7.marketOrdersEnabled !== void 0) {
												if (typeof data7.marketOrdersEnabled !== "boolean") {
													validate156.errors = [{
														instancePath: instancePath + "/policy/marketOrdersEnabled",
														schemaPath: "#/$defs/RiskPolicy/properties/marketOrdersEnabled/type",
														keyword: "type",
														params: { type: "boolean" },
														message: "must be boolean"
													}];
													return false;
												}
												var valid5 = true;
											} else var valid5 = true;
											if (valid5) {
												if (data7.maxDailyRealizedLoss !== void 0) {
													let data10 = data7.maxDailyRealizedLoss;
													if (typeof data10 !== "string" && data10 !== null) {
														validate156.errors = [{
															instancePath: instancePath + "/policy/maxDailyRealizedLoss",
															schemaPath: "#/$defs/RiskPolicy/properties/maxDailyRealizedLoss/type",
															keyword: "type",
															params: { type: schema72.properties.maxDailyRealizedLoss.type },
															message: "must be string,null"
														}];
														return false;
													}
													if (typeof data10 === "string") {
														if (func1(data10) > 32) {
															validate156.errors = [{
																instancePath: instancePath + "/policy/maxDailyRealizedLoss",
																schemaPath: "#/$defs/RiskPolicy/properties/maxDailyRealizedLoss/maxLength",
																keyword: "maxLength",
																params: { limit: 32 },
																message: "must NOT have more than 32 characters"
															}];
															return false;
														}
													}
													var valid5 = true;
												} else var valid5 = true;
												if (valid5) {
													if (data7.maxDailyTradedNotional !== void 0) {
														let data11 = data7.maxDailyTradedNotional;
														if (typeof data11 !== "string" && data11 !== null) {
															validate156.errors = [{
																instancePath: instancePath + "/policy/maxDailyTradedNotional",
																schemaPath: "#/$defs/RiskPolicy/properties/maxDailyTradedNotional/type",
																keyword: "type",
																params: { type: schema72.properties.maxDailyTradedNotional.type },
																message: "must be string,null"
															}];
															return false;
														}
														if (typeof data11 === "string") {
															if (func1(data11) > 32) {
																validate156.errors = [{
																	instancePath: instancePath + "/policy/maxDailyTradedNotional",
																	schemaPath: "#/$defs/RiskPolicy/properties/maxDailyTradedNotional/maxLength",
																	keyword: "maxLength",
																	params: { limit: 32 },
																	message: "must NOT have more than 32 characters"
																}];
																return false;
															}
														}
														var valid5 = true;
													} else var valid5 = true;
													if (valid5) {
														if (data7.maxOrderNotional !== void 0) {
															let data12 = data7.maxOrderNotional;
															if (typeof data12 !== "string" && data12 !== null) {
																validate156.errors = [{
																	instancePath: instancePath + "/policy/maxOrderNotional",
																	schemaPath: "#/$defs/RiskPolicy/properties/maxOrderNotional/type",
																	keyword: "type",
																	params: { type: schema72.properties.maxOrderNotional.type },
																	message: "must be string,null"
																}];
																return false;
															}
															if (typeof data12 === "string") {
																if (func1(data12) > 32) {
																	validate156.errors = [{
																		instancePath: instancePath + "/policy/maxOrderNotional",
																		schemaPath: "#/$defs/RiskPolicy/properties/maxOrderNotional/maxLength",
																		keyword: "maxLength",
																		params: { limit: 32 },
																		message: "must NOT have more than 32 characters"
																	}];
																	return false;
																}
															}
															var valid5 = true;
														} else var valid5 = true;
														if (valid5) {
															if (data7.maxSingleInstrumentExposurePercent !== void 0) {
																let data13 = data7.maxSingleInstrumentExposurePercent;
																if (typeof data13 !== "string" && data13 !== null) {
																	validate156.errors = [{
																		instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																		schemaPath: "#/$defs/RiskPolicy/properties/maxSingleInstrumentExposurePercent/type",
																		keyword: "type",
																		params: { type: schema72.properties.maxSingleInstrumentExposurePercent.type },
																		message: "must be string,null"
																	}];
																	return false;
																}
																if (typeof data13 === "string") {
																	if (func1(data13) > 32) {
																		validate156.errors = [{
																			instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																			schemaPath: "#/$defs/RiskPolicy/properties/maxSingleInstrumentExposurePercent/maxLength",
																			keyword: "maxLength",
																			params: { limit: 32 },
																			message: "must NOT have more than 32 characters"
																		}];
																		return false;
																	}
																}
																var valid5 = true;
															} else var valid5 = true;
															if (valid5) {
																if (data7.staleQuoteThresholdSeconds !== void 0) {
																	let data14 = data7.staleQuoteThresholdSeconds;
																	if (!(typeof data14 == "number" && !(data14 % 1) && !isNaN(data14))) {
																		validate156.errors = [{
																			instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																			schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/type",
																			keyword: "type",
																			params: { type: "integer" },
																			message: "must be integer"
																		}];
																		return false;
																	}
																	if (typeof data14 == "number") {
																		if (data14 > 86400 || isNaN(data14)) {
																			validate156.errors = [{
																				instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																				schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/maximum",
																				keyword: "maximum",
																				params: {
																					comparison: "<=",
																					limit: 86400
																				},
																				message: "must be <= 86400"
																			}];
																			return false;
																		} else if (data14 < 1 || isNaN(data14)) {
																			validate156.errors = [{
																				instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																				schemaPath: "#/$defs/RiskPolicy/properties/staleQuoteThresholdSeconds/minimum",
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
																	var valid5 = true;
																} else var valid5 = true;
															}
														}
													}
												}
											}
										}
									} else {
										validate156.errors = [{
											instancePath: instancePath + "/policy",
											schemaPath: "#/$defs/RiskPolicy/type",
											keyword: "type",
											params: { type: "object" },
											message: "must be object"
										}];
										return false;
									}
									var valid0 = true;
								} else var valid0 = true;
								if (valid0) {
									if (data.policyVersion !== void 0) {
										let data15 = data.policyVersion;
										if (!(typeof data15 == "number" && !(data15 % 1) && !isNaN(data15))) {
											validate156.errors = [{
												instancePath: instancePath + "/policyVersion",
												schemaPath: "#/properties/policyVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data15 == "number") {
											if (data15 < 1 || isNaN(data15)) {
												validate156.errors = [{
													instancePath: instancePath + "/policyVersion",
													schemaPath: "#/properties/policyVersion/minimum",
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
									if (valid0) {
										if (data.stateVersion !== void 0) {
											let data16 = data.stateVersion;
											if (typeof data16 === "string") {
												if (func1(data16) > 256) {
													validate156.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/maxLength",
														keyword: "maxLength",
														params: { limit: 256 },
														message: "must NOT have more than 256 characters"
													}];
													return false;
												} else if (func1(data16) < 1) {
													validate156.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate156.errors = [{
													instancePath: instancePath + "/stateVersion",
													schemaPath: "#/properties/stateVersion/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												}];
												return false;
											}
											var valid0 = true;
										} else var valid0 = true;
										if (valid0) {
											if (data.updatedAt !== void 0) {
												if (typeof data.updatedAt !== "string") {
													validate156.errors = [{
														instancePath: instancePath + "/updatedAt",
														schemaPath: "#/properties/updatedAt/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.workspaceId !== void 0) {
													let data18 = data.workspaceId;
													if (typeof data18 === "string") {
														if (func1(data18) > 128) {
															validate156.errors = [{
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/properties/workspaceId/maxLength",
																keyword: "maxLength",
																params: { limit: 128 },
																message: "must NOT have more than 128 characters"
															}];
															return false;
														} else if (func1(data18) < 1) {
															validate156.errors = [{
																instancePath: instancePath + "/workspaceId",
																schemaPath: "#/properties/workspaceId/minLength",
																keyword: "minLength",
																params: { limit: 1 },
																message: "must NOT have fewer than 1 characters"
															}];
															return false;
														}
													} else {
														validate156.errors = [{
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
				}
			}
		} else {
			validate156.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate156.errors = vErrors;
		return true;
	}
	validate156.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RiskQuery = validate157;
	function validate157(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate157.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId")) {
				validate157.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "workspaceId")) {
					validate157.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.workspaceId !== void 0) {
					let data0 = data.workspaceId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate157.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate157.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate157.errors = [{
							instancePath: instancePath + "/workspaceId",
							schemaPath: "#/properties/workspaceId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
				}
			}
		} else {
			validate157.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate157.errors = vErrors;
		return true;
	}
	validate157.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RuntimeComponent = validate158;
	function validate158(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate158.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.status === void 0 && (missing0 = "status") || data.message === void 0 && (missing0 = "message")) {
				validate158.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "message" || key0 === "status")) {
					validate158.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.id !== void 0) {
					let data0 = data.id;
					if (typeof data0 === "string") {
						if (func1(data0) < 1) {
							validate158.errors = [{
								instancePath: instancePath + "/id",
								schemaPath: "#/properties/id/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate158.errors = [{
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
							validate158.errors = [{
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
								validate158.errors = [{
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
			validate158.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate158.errors = vErrors;
		return true;
	}
	validate158.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RuntimeStatus = validate159;
	function validate159(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate159.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate159.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate159.errors = [{
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
									validate159.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate159.errors = [{
											instancePath: instancePath + "/components/" + i0,
											schemaPath: "#/$defs/RuntimeComponent/additionalProperties",
											keyword: "additionalProperties",
											params: { additionalProperty: key1 },
											message: "must NOT have additional properties"
										}];
										return false;
									}
									if (data1.id !== void 0) {
										let data2 = data1.id;
										if (typeof data2 === "string") {
											if (func1(data2) < 1) {
												validate159.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/id",
													schemaPath: "#/$defs/RuntimeComponent/properties/id/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate159.errors = [{
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
												validate159.errors = [{
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
													validate159.errors = [{
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
								validate159.errors = [{
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
						validate159.errors = [{
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
							validate159.errors = [{
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
								validate159.errors = [{
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
			validate159.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate159.errors = vErrors;
		return true;
	}
	validate159.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SaveRiskPolicy = validate160;
	function validate160(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate160.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.policy === void 0 && (missing0 = "policy")) {
				validate160.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "policy" || key0 === "workspaceId")) {
					validate160.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.expectedStateVersion !== void 0) {
					let data0 = data.expectedStateVersion;
					if (typeof data0 === "string") {
						if (func1(data0) > 256) {
							validate160.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/maxLength",
								keyword: "maxLength",
								params: { limit: 256 },
								message: "must NOT have more than 256 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate160.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate160.errors = [{
							instancePath: instancePath + "/expectedStateVersion",
							schemaPath: "#/properties/expectedStateVersion/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.policy !== void 0) {
						let data1 = data.policy;
						if (data1 && typeof data1 == "object" && !Array.isArray(data1)) {
							let missing1;
							if (data1.maxOrderNotional === void 0 && (missing1 = "maxOrderNotional") || data1.maxSingleInstrumentExposurePercent === void 0 && (missing1 = "maxSingleInstrumentExposurePercent") || data1.maxDailyTradedNotional === void 0 && (missing1 = "maxDailyTradedNotional") || data1.maxDailyRealizedLoss === void 0 && (missing1 = "maxDailyRealizedLoss") || data1.staleQuoteThresholdSeconds === void 0 && (missing1 = "staleQuoteThresholdSeconds") || data1.marketOrdersEnabled === void 0 && (missing1 = "marketOrdersEnabled") || data1.liveInactivityTimeoutMinutes === void 0 && (missing1 = "liveInactivityTimeoutMinutes")) {
								validate160.errors = [{
									instancePath: instancePath + "/policy",
									schemaPath: "#/$defs/RiskPolicyInput/required",
									keyword: "required",
									params: { missingProperty: missing1 },
									message: "must have required property '" + missing1 + "'"
								}];
								return false;
							} else {
								for (const key1 in data1) if (!(key1 === "liveInactivityTimeoutMinutes" || key1 === "marketOrdersEnabled" || key1 === "maxDailyRealizedLoss" || key1 === "maxDailyTradedNotional" || key1 === "maxOrderNotional" || key1 === "maxSingleInstrumentExposurePercent" || key1 === "staleQuoteThresholdSeconds")) {
									validate160.errors = [{
										instancePath: instancePath + "/policy",
										schemaPath: "#/$defs/RiskPolicyInput/additionalProperties",
										keyword: "additionalProperties",
										params: { additionalProperty: key1 },
										message: "must NOT have additional properties"
									}];
									return false;
								}
								if (data1.liveInactivityTimeoutMinutes !== void 0) {
									let data2 = data1.liveInactivityTimeoutMinutes;
									if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
										validate160.errors = [{
											instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
											schemaPath: "#/$defs/RiskPolicyInput/properties/liveInactivityTimeoutMinutes/type",
											keyword: "type",
											params: { type: "integer" },
											message: "must be integer"
										}];
										return false;
									}
									if (typeof data2 == "number") {
										if (data2 > 1440 || isNaN(data2)) {
											validate160.errors = [{
												instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
												schemaPath: "#/$defs/RiskPolicyInput/properties/liveInactivityTimeoutMinutes/maximum",
												keyword: "maximum",
												params: {
													comparison: "<=",
													limit: 1440
												},
												message: "must be <= 1440"
											}];
											return false;
										} else if (data2 < 1 || isNaN(data2)) {
											validate160.errors = [{
												instancePath: instancePath + "/policy/liveInactivityTimeoutMinutes",
												schemaPath: "#/$defs/RiskPolicyInput/properties/liveInactivityTimeoutMinutes/minimum",
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
									var valid2 = true;
								} else var valid2 = true;
								if (valid2) {
									if (data1.marketOrdersEnabled !== void 0) {
										if (typeof data1.marketOrdersEnabled !== "boolean") {
											validate160.errors = [{
												instancePath: instancePath + "/policy/marketOrdersEnabled",
												schemaPath: "#/$defs/RiskPolicyInput/properties/marketOrdersEnabled/type",
												keyword: "type",
												params: { type: "boolean" },
												message: "must be boolean"
											}];
											return false;
										}
										var valid2 = true;
									} else var valid2 = true;
									if (valid2) {
										if (data1.maxDailyRealizedLoss !== void 0) {
											let data4 = data1.maxDailyRealizedLoss;
											if (typeof data4 !== "string" && data4 !== null) {
												validate160.errors = [{
													instancePath: instancePath + "/policy/maxDailyRealizedLoss",
													schemaPath: "#/$defs/RiskPolicyInput/properties/maxDailyRealizedLoss/type",
													keyword: "type",
													params: { type: schema96.properties.maxDailyRealizedLoss.type },
													message: "must be string,null"
												}];
												return false;
											}
											if (typeof data4 === "string") {
												if (func1(data4) > 32) {
													validate160.errors = [{
														instancePath: instancePath + "/policy/maxDailyRealizedLoss",
														schemaPath: "#/$defs/RiskPolicyInput/properties/maxDailyRealizedLoss/maxLength",
														keyword: "maxLength",
														params: { limit: 32 },
														message: "must NOT have more than 32 characters"
													}];
													return false;
												}
											}
											var valid2 = true;
										} else var valid2 = true;
										if (valid2) {
											if (data1.maxDailyTradedNotional !== void 0) {
												let data5 = data1.maxDailyTradedNotional;
												if (typeof data5 !== "string" && data5 !== null) {
													validate160.errors = [{
														instancePath: instancePath + "/policy/maxDailyTradedNotional",
														schemaPath: "#/$defs/RiskPolicyInput/properties/maxDailyTradedNotional/type",
														keyword: "type",
														params: { type: schema96.properties.maxDailyTradedNotional.type },
														message: "must be string,null"
													}];
													return false;
												}
												if (typeof data5 === "string") {
													if (func1(data5) > 32) {
														validate160.errors = [{
															instancePath: instancePath + "/policy/maxDailyTradedNotional",
															schemaPath: "#/$defs/RiskPolicyInput/properties/maxDailyTradedNotional/maxLength",
															keyword: "maxLength",
															params: { limit: 32 },
															message: "must NOT have more than 32 characters"
														}];
														return false;
													}
												}
												var valid2 = true;
											} else var valid2 = true;
											if (valid2) {
												if (data1.maxOrderNotional !== void 0) {
													let data6 = data1.maxOrderNotional;
													if (typeof data6 !== "string" && data6 !== null) {
														validate160.errors = [{
															instancePath: instancePath + "/policy/maxOrderNotional",
															schemaPath: "#/$defs/RiskPolicyInput/properties/maxOrderNotional/type",
															keyword: "type",
															params: { type: schema96.properties.maxOrderNotional.type },
															message: "must be string,null"
														}];
														return false;
													}
													if (typeof data6 === "string") {
														if (func1(data6) > 32) {
															validate160.errors = [{
																instancePath: instancePath + "/policy/maxOrderNotional",
																schemaPath: "#/$defs/RiskPolicyInput/properties/maxOrderNotional/maxLength",
																keyword: "maxLength",
																params: { limit: 32 },
																message: "must NOT have more than 32 characters"
															}];
															return false;
														}
													}
													var valid2 = true;
												} else var valid2 = true;
												if (valid2) {
													if (data1.maxSingleInstrumentExposurePercent !== void 0) {
														let data7 = data1.maxSingleInstrumentExposurePercent;
														if (typeof data7 !== "string" && data7 !== null) {
															validate160.errors = [{
																instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																schemaPath: "#/$defs/RiskPolicyInput/properties/maxSingleInstrumentExposurePercent/type",
																keyword: "type",
																params: { type: schema96.properties.maxSingleInstrumentExposurePercent.type },
																message: "must be string,null"
															}];
															return false;
														}
														if (typeof data7 === "string") {
															if (func1(data7) > 32) {
																validate160.errors = [{
																	instancePath: instancePath + "/policy/maxSingleInstrumentExposurePercent",
																	schemaPath: "#/$defs/RiskPolicyInput/properties/maxSingleInstrumentExposurePercent/maxLength",
																	keyword: "maxLength",
																	params: { limit: 32 },
																	message: "must NOT have more than 32 characters"
																}];
																return false;
															}
														}
														var valid2 = true;
													} else var valid2 = true;
													if (valid2) {
														if (data1.staleQuoteThresholdSeconds !== void 0) {
															let data8 = data1.staleQuoteThresholdSeconds;
															if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8))) {
																validate160.errors = [{
																	instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																	schemaPath: "#/$defs/RiskPolicyInput/properties/staleQuoteThresholdSeconds/type",
																	keyword: "type",
																	params: { type: "integer" },
																	message: "must be integer"
																}];
																return false;
															}
															if (typeof data8 == "number") {
																if (data8 > 86400 || isNaN(data8)) {
																	validate160.errors = [{
																		instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																		schemaPath: "#/$defs/RiskPolicyInput/properties/staleQuoteThresholdSeconds/maximum",
																		keyword: "maximum",
																		params: {
																			comparison: "<=",
																			limit: 86400
																		},
																		message: "must be <= 86400"
																	}];
																	return false;
																} else if (data8 < 1 || isNaN(data8)) {
																	validate160.errors = [{
																		instancePath: instancePath + "/policy/staleQuoteThresholdSeconds",
																		schemaPath: "#/$defs/RiskPolicyInput/properties/staleQuoteThresholdSeconds/minimum",
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
							validate160.errors = [{
								instancePath: instancePath + "/policy",
								schemaPath: "#/$defs/RiskPolicyInput/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data9 = data.workspaceId;
							if (typeof data9 === "string") {
								if (func1(data9) > 128) {
									validate160.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data9) < 1) {
									validate160.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate160.errors = [{
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
		} else {
			validate160.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate160.errors = vErrors;
		return true;
	}
	validate160.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SetDefaultModel = validate161;
	function validate161(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate161.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate161.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "modelId" || key0 === "provider" || key0 === "thinkingType" || key0 === "workspaceId")) {
						validate161.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.expectedStateVersion !== void 0) {
							let data0 = data.expectedStateVersion;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 256) {
										validate161.errors = [{
											instancePath: instancePath + "/expectedStateVersion",
											schemaPath: "#/properties/expectedStateVersion/maxLength",
											keyword: "maxLength",
											params: { limit: 256 },
											message: "must NOT have more than 256 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate161.errors = [{
											instancePath: instancePath + "/expectedStateVersion",
											schemaPath: "#/properties/expectedStateVersion/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate161.errors = [{
										instancePath: instancePath + "/expectedStateVersion",
										schemaPath: "#/properties/expectedStateVersion/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.modelId !== void 0) {
								let data1 = data.modelId;
								const _errs4 = errors;
								if (errors === _errs4) {
									if (typeof data1 === "string") {
										if (func1(data1) > 128) {
											validate161.errors = [{
												instancePath: instancePath + "/modelId",
												schemaPath: "#/properties/modelId/maxLength",
												keyword: "maxLength",
												params: { limit: 128 },
												message: "must NOT have more than 128 characters"
											}];
											return false;
										} else if (func1(data1) < 1) {
											validate161.errors = [{
												instancePath: instancePath + "/modelId",
												schemaPath: "#/properties/modelId/minLength",
												keyword: "minLength",
												params: { limit: 1 },
												message: "must NOT have fewer than 1 characters"
											}];
											return false;
										}
									} else {
										validate161.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.provider !== void 0) {
									let data2 = data.provider;
									const _errs6 = errors;
									if (typeof data2 !== "string") {
										validate161.errors = [{
											instancePath: instancePath + "/provider",
											schemaPath: "#/$defs/ModelProvider/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									if (!(data2 === "CHATGPT" || data2 === "DEEPSEEK")) {
										validate161.errors = [{
											instancePath: instancePath + "/provider",
											schemaPath: "#/$defs/ModelProvider/enum",
											keyword: "enum",
											params: { allowedValues: schema49.enum },
											message: "must be equal to one of the allowed values"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.thinkingType !== void 0) {
										let data3 = data.thinkingType;
										const _errs9 = errors;
										const _errs10 = errors;
										let valid2 = false;
										const _errs11 = errors;
										if (typeof data3 !== "string") {
											const err0 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/$defs/ThinkingType/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										if (!(data3 === "disabled" || data3 === "enabled")) {
											const err1 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/$defs/ThinkingType/enum",
												keyword: "enum",
												params: { allowedValues: schema51.enum },
												message: "must be equal to one of the allowed values"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										const _errs14 = errors;
										if (data3 !== null) {
											const err2 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/properties/thinkingType/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err2];
											else vErrors.push(err2);
											errors++;
										}
										var _valid0 = _errs14 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err3 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/properties/thinkingType/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err3];
											else vErrors.push(err3);
											errors++;
											validate161.errors = vErrors;
											return false;
										} else {
											errors = _errs10;
											if (vErrors !== null) {
												if (_errs10) vErrors.length = _errs10;
												else vErrors = null;
											}
										}
										var valid0 = _errs9 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.workspaceId !== void 0) {
											let data4 = data.workspaceId;
											const _errs16 = errors;
											if (errors === _errs16) {
												if (typeof data4 === "string") {
													if (func1(data4) > 128) {
														validate161.errors = [{
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/properties/workspaceId/maxLength",
															keyword: "maxLength",
															params: { limit: 128 },
															message: "must NOT have more than 128 characters"
														}];
														return false;
													} else if (func1(data4) < 1) {
														validate161.errors = [{
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/properties/workspaceId/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate161.errors = [{
														instancePath: instancePath + "/workspaceId",
														schemaPath: "#/properties/workspaceId/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											}
											var valid0 = _errs16 === errors;
										} else var valid0 = true;
									}
								}
							}
						}
					}
				}
			} else {
				validate161.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate161.errors = vErrors;
		return errors === 0;
	}
	validate161.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SetFallbackPolicy = validate162;
	function validate162(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate162.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.automaticFallback === void 0 && (missing0 = "automaticFallback")) {
				validate162.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "automaticFallback" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate162.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.automaticFallback !== void 0) {
					if (typeof data.automaticFallback !== "boolean") {
						validate162.errors = [{
							instancePath: instancePath + "/automaticFallback",
							schemaPath: "#/properties/automaticFallback/type",
							keyword: "type",
							params: { type: "boolean" },
							message: "must be boolean"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.expectedStateVersion !== void 0) {
						let data1 = data.expectedStateVersion;
						if (typeof data1 === "string") {
							if (func1(data1) > 256) {
								validate162.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate162.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate162.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							}];
							return false;
						}
						var valid0 = true;
					} else var valid0 = true;
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data2 = data.workspaceId;
							if (typeof data2 === "string") {
								if (func1(data2) > 128) {
									validate162.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate162.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate162.errors = [{
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
		} else {
			validate162.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate162.errors = vErrors;
		return true;
	}
	validate162.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SetOnboardingStep = validate163;
	function validate163(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate163.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.step === void 0 && (missing0 = "step")) {
				validate163.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "step" || key0 === "workspaceId")) {
					validate163.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.expectedStateVersion !== void 0) {
					let data0 = data.expectedStateVersion;
					if (typeof data0 === "string") {
						if (func1(data0) > 256) {
							validate163.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/maxLength",
								keyword: "maxLength",
								params: { limit: 256 },
								message: "must NOT have more than 256 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate163.errors = [{
								instancePath: instancePath + "/expectedStateVersion",
								schemaPath: "#/properties/expectedStateVersion/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate163.errors = [{
							instancePath: instancePath + "/expectedStateVersion",
							schemaPath: "#/properties/expectedStateVersion/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					var valid0 = true;
				} else var valid0 = true;
				if (valid0) {
					if (data.step !== void 0) {
						let data1 = data.step;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1))) {
							validate163.errors = [{
								instancePath: instancePath + "/step",
								schemaPath: "#/properties/step/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							}];
							return false;
						}
						if (typeof data1 == "number") {
							if (data1 > 5 || isNaN(data1)) {
								validate163.errors = [{
									instancePath: instancePath + "/step",
									schemaPath: "#/properties/step/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 5
									},
									message: "must be <= 5"
								}];
								return false;
							} else if (data1 < 1 || isNaN(data1)) {
								validate163.errors = [{
									instancePath: instancePath + "/step",
									schemaPath: "#/properties/step/minimum",
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
					if (valid0) {
						if (data.workspaceId !== void 0) {
							let data2 = data.workspaceId;
							if (typeof data2 === "string") {
								if (func1(data2) > 128) {
									validate163.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate163.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate163.errors = [{
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
		} else {
			validate163.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate163.errors = vErrors;
		return true;
	}
	validate163.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Snapshot = validate164;
	function validate164(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate164.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
					validate164.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "projection")) {
						validate164.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.aggregateId !== void 0) {
							let data0 = data.aggregateId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate164.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate164.errors = [{
										instancePath: instancePath + "/aggregateId",
										schemaPath: "#/properties/aggregateId/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.aggregateType !== void 0) {
								let data1 = data.aggregateType;
								const _errs4 = errors;
								if (typeof data1 !== "string") {
									validate164.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway" || data1 === "model" || data1 === "risk")) {
									validate164.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema82.properties.aggregateType.enum },
										message: "must be equal to one of the allowed values"
									}];
									return false;
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.lastSequence !== void 0) {
									let data2 = data.lastSequence;
									const _errs6 = errors;
									if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2))) {
										validate164.errors = [{
											instancePath: instancePath + "/lastSequence",
											schemaPath: "#/properties/lastSequence/type",
											keyword: "type",
											params: { type: "integer" },
											message: "must be integer"
										}];
										return false;
									}
									if (errors === _errs6) {
										if (typeof data2 == "number") {
											if (data2 > 9007199254740991 || isNaN(data2)) {
												validate164.errors = [{
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
												validate164.errors = [{
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
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.projection !== void 0) {
										const _errs8 = errors;
										if (!validate24(data.projection, {
											instancePath: instancePath + "/projection",
											parentData: data,
											parentDataProperty: "projection",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
											errors = vErrors.length;
										}
										var valid0 = _errs8 === errors;
									} else var valid0 = true;
								}
							}
						}
					}
				}
			} else {
				validate164.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate164.errors = vErrors;
		return errors === 0;
	}
	validate164.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Subscribe = validate166;
	function validate166(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate166.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence")) {
				validate166.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType")) {
					validate166.errors = [{
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
						validate166.errors = [{
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
							validate166.errors = [{
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
							validate166.errors = [{
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
						let data1 = data.aggregateId;
						if (typeof data1 === "string") {
							if (func1(data1) > 128) {
								validate166.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate166.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate166.errors = [{
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
							let data2 = data.aggregateType;
							if (typeof data2 === "string") {
								if (func1(data2) > 64) {
									validate166.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/maxLength",
										keyword: "maxLength",
										params: { limit: 64 },
										message: "must NOT have more than 64 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate166.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate166.errors = [{
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
			validate166.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate166.errors = vErrors;
		return true;
	}
	validate166.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SubscriptionAck = validate167;
	function validate167(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate167.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence") || data.lastSequence === void 0 && (missing0 = "lastSequence") || data.replayedCount === void 0 && (missing0 = "replayedCount")) {
				validate167.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "replayedCount")) {
					validate167.errors = [{
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
						validate167.errors = [{
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
							validate167.errors = [{
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
							validate167.errors = [{
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
						let data1 = data.aggregateId;
						if (typeof data1 === "string") {
							if (func1(data1) < 1) {
								validate167.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate167.errors = [{
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
							let data2 = data.aggregateType;
							if (typeof data2 !== "string") {
								validate167.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if (!(data2 === "workspace" || data2 === "account" || data2 === "model-gateway" || data2 === "model" || data2 === "risk")) {
								validate167.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/enum",
									keyword: "enum",
									params: { allowedValues: schema85.properties.aggregateType.enum },
									message: "must be equal to one of the allowed values"
								}];
								return false;
							}
							var valid0 = true;
						} else var valid0 = true;
						if (valid0) {
							if (data.lastSequence !== void 0) {
								let data3 = data.lastSequence;
								if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
									validate167.errors = [{
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
										validate167.errors = [{
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
										validate167.errors = [{
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
										validate167.errors = [{
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
											validate167.errors = [{
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
											validate167.errors = [{
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
			validate167.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate167.errors = vErrors;
		return true;
	}
	validate167.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SuccessEnvelope = validate168;
	function validate168(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate168.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate168.errors = [{
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
						validate168.errors = [{
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
							if (!validate51(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate168.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate168.errors = [{
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
									let data2 = data.requestId;
									const _errs5 = errors;
									if (errors === _errs5) {
										if (typeof data2 === "string") {
											if (func1(data2) > 128) {
												validate168.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate168.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate168.errors = [{
												instancePath: instancePath + "/requestId",
												schemaPath: "#/properties/requestId/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									}
									var valid0 = _errs5 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.schemaVersion !== void 0) {
										let data3 = data.schemaVersion;
										const _errs7 = errors;
										if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3))) {
											validate168.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate168.errors = [{
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
													validate168.errors = [{
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
											if (errors === _errs9) {
												if (typeof data4 === "string") {
													if (func1(data4) < 1) {
														validate168.errors = [{
															instancePath: instancePath + "/stateVersion",
															schemaPath: "#/properties/stateVersion/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate168.errors = [{
														instancePath: instancePath + "/stateVersion",
														schemaPath: "#/properties/stateVersion/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
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
				validate168.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate168.errors = vErrors;
		return errors === 0;
	}
	validate168.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ThinkingType = validate170;
	function validate170(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate170.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate170.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "disabled" || data === "enabled")) {
			validate170.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema51.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate170.errors = vErrors;
		return true;
	}
	validate170.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.TradeXError = validate171;
	function validate171(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate171.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate171.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate171.errors = [{
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
						validate171.errors = [{
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
							validate171.errors = [{
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
								validate171.errors = [{
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
									validate171.errors = [{
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
													validate171.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate171.errors = [{
															instancePath: instancePath + "/remediationActions/" + i0,
															schemaPath: "#/$defs/Remediation/additionalProperties",
															keyword: "additionalProperties",
															params: { additionalProperty: key1 },
															message: "must NOT have additional properties"
														}];
														return false;
													}
													if (data5.id !== void 0) {
														let data6 = data5.id;
														if (typeof data6 === "string") {
															if (func1(data6) < 1) {
																validate171.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																	schemaPath: "#/$defs/Remediation/properties/id/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																}];
																return false;
															}
														} else {
															validate171.errors = [{
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
																validate171.errors = [{
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
												validate171.errors = [{
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
										validate171.errors = [{
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
											validate171.errors = [{
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
			validate171.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate171.errors = vErrors;
		return true;
	}
	validate171.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.VerifyRoute = validate172;
	function validate172(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate172.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.provider === void 0 && (missing0 = "provider") || data.modelId === void 0 && (missing0 = "modelId")) {
					validate172.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!(key0 === "expectedStateVersion" || key0 === "modelId" || key0 === "provider" || key0 === "thinkingType" || key0 === "workspaceId")) {
						validate172.errors = [{
							instancePath,
							schemaPath: "#/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key0 },
							message: "must NOT have additional properties"
						}];
						return false;
					}
					if (_errs1 === errors) {
						if (data.expectedStateVersion !== void 0) {
							let data0 = data.expectedStateVersion;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) > 256) {
										validate172.errors = [{
											instancePath: instancePath + "/expectedStateVersion",
											schemaPath: "#/properties/expectedStateVersion/maxLength",
											keyword: "maxLength",
											params: { limit: 256 },
											message: "must NOT have more than 256 characters"
										}];
										return false;
									} else if (func1(data0) < 1) {
										validate172.errors = [{
											instancePath: instancePath + "/expectedStateVersion",
											schemaPath: "#/properties/expectedStateVersion/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate172.errors = [{
										instancePath: instancePath + "/expectedStateVersion",
										schemaPath: "#/properties/expectedStateVersion/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.modelId !== void 0) {
								let data1 = data.modelId;
								const _errs4 = errors;
								if (errors === _errs4) {
									if (typeof data1 === "string") {
										if (func1(data1) > 128) {
											validate172.errors = [{
												instancePath: instancePath + "/modelId",
												schemaPath: "#/properties/modelId/maxLength",
												keyword: "maxLength",
												params: { limit: 128 },
												message: "must NOT have more than 128 characters"
											}];
											return false;
										} else if (func1(data1) < 1) {
											validate172.errors = [{
												instancePath: instancePath + "/modelId",
												schemaPath: "#/properties/modelId/minLength",
												keyword: "minLength",
												params: { limit: 1 },
												message: "must NOT have fewer than 1 characters"
											}];
											return false;
										}
									} else {
										validate172.errors = [{
											instancePath: instancePath + "/modelId",
											schemaPath: "#/properties/modelId/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
								}
								var valid0 = _errs4 === errors;
							} else var valid0 = true;
							if (valid0) {
								if (data.provider !== void 0) {
									let data2 = data.provider;
									const _errs6 = errors;
									if (typeof data2 !== "string") {
										validate172.errors = [{
											instancePath: instancePath + "/provider",
											schemaPath: "#/$defs/ModelProvider/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										}];
										return false;
									}
									if (!(data2 === "CHATGPT" || data2 === "DEEPSEEK")) {
										validate172.errors = [{
											instancePath: instancePath + "/provider",
											schemaPath: "#/$defs/ModelProvider/enum",
											keyword: "enum",
											params: { allowedValues: schema49.enum },
											message: "must be equal to one of the allowed values"
										}];
										return false;
									}
									var valid0 = _errs6 === errors;
								} else var valid0 = true;
								if (valid0) {
									if (data.thinkingType !== void 0) {
										let data3 = data.thinkingType;
										const _errs9 = errors;
										const _errs10 = errors;
										let valid2 = false;
										const _errs11 = errors;
										if (typeof data3 !== "string") {
											const err0 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/$defs/ThinkingType/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											if (vErrors === null) vErrors = [err0];
											else vErrors.push(err0);
											errors++;
										}
										if (!(data3 === "disabled" || data3 === "enabled")) {
											const err1 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/$defs/ThinkingType/enum",
												keyword: "enum",
												params: { allowedValues: schema51.enum },
												message: "must be equal to one of the allowed values"
											};
											if (vErrors === null) vErrors = [err1];
											else vErrors.push(err1);
											errors++;
										}
										var _valid0 = _errs11 === errors;
										valid2 = valid2 || _valid0;
										const _errs14 = errors;
										if (data3 !== null) {
											const err2 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/properties/thinkingType/anyOf/1/type",
												keyword: "type",
												params: { type: "null" },
												message: "must be null"
											};
											if (vErrors === null) vErrors = [err2];
											else vErrors.push(err2);
											errors++;
										}
										var _valid0 = _errs14 === errors;
										valid2 = valid2 || _valid0;
										if (!valid2) {
											const err3 = {
												instancePath: instancePath + "/thinkingType",
												schemaPath: "#/properties/thinkingType/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											if (vErrors === null) vErrors = [err3];
											else vErrors.push(err3);
											errors++;
											validate172.errors = vErrors;
											return false;
										} else {
											errors = _errs10;
											if (vErrors !== null) {
												if (_errs10) vErrors.length = _errs10;
												else vErrors = null;
											}
										}
										var valid0 = _errs9 === errors;
									} else var valid0 = true;
									if (valid0) {
										if (data.workspaceId !== void 0) {
											let data4 = data.workspaceId;
											const _errs16 = errors;
											if (errors === _errs16) {
												if (typeof data4 === "string") {
													if (func1(data4) > 128) {
														validate172.errors = [{
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/properties/workspaceId/maxLength",
															keyword: "maxLength",
															params: { limit: 128 },
															message: "must NOT have more than 128 characters"
														}];
														return false;
													} else if (func1(data4) < 1) {
														validate172.errors = [{
															instancePath: instancePath + "/workspaceId",
															schemaPath: "#/properties/workspaceId/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate172.errors = [{
														instancePath: instancePath + "/workspaceId",
														schemaPath: "#/properties/workspaceId/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											}
											var valid0 = _errs16 === errors;
										} else var valid0 = true;
									}
								}
							}
						}
					}
				}
			} else {
				validate172.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate172.errors = vErrors;
		return errors === 0;
	}
	validate172.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Workspace = validate173;
	function validate173(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate173.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.name === void 0 && (missing0 = "name") || data.baseCurrency === void 0 && (missing0 = "baseCurrency") || data.path === void 0 && (missing0 = "path") || data.createdAt === void 0 && (missing0 = "createdAt") || data.lastOpenedAt === void 0 && (missing0 = "lastOpenedAt") || data.storageSchemaVersion === void 0 && (missing0 = "storageSchemaVersion")) {
				validate173.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "createdAt" || key0 === "lastOpenedAt" || key0 === "name" || key0 === "path" || key0 === "storageSchemaVersion" || key0 === "workspaceId")) {
					validate173.errors = [{
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
						validate173.errors = [{
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
							validate173.errors = [{
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
								validate173.errors = [{
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
									validate173.errors = [{
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
										validate173.errors = [{
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
											validate173.errors = [{
												instancePath: instancePath + "/storageSchemaVersion",
												schemaPath: "#/properties/storageSchemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data5 == "number") {
											if (data5 > 5 || isNaN(data5)) {
												validate173.errors = [{
													instancePath: instancePath + "/storageSchemaVersion",
													schemaPath: "#/properties/storageSchemaVersion/maximum",
													keyword: "maximum",
													params: {
														comparison: "<=",
														limit: 5
													},
													message: "must be <= 5"
												}];
												return false;
											} else if (data5 < 1 || isNaN(data5)) {
												validate173.errors = [{
													instancePath: instancePath + "/storageSchemaVersion",
													schemaPath: "#/properties/storageSchemaVersion/minimum",
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
									if (valid0) {
										if (data.workspaceId !== void 0) {
											let data6 = data.workspaceId;
											if (typeof data6 === "string") {
												if (func1(data6) < 1) {
													validate173.errors = [{
														instancePath: instancePath + "/workspaceId",
														schemaPath: "#/properties/workspaceId/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate173.errors = [{
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
			validate173.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate173.errors = vErrors;
		return true;
	}
	validate173.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.WorkspaceQuery = validate174;
	function validate174(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate174.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId")) {
				validate174.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "workspaceId")) {
					validate174.errors = [{
						instancePath,
						schemaPath: "#/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key0 },
						message: "must NOT have additional properties"
					}];
					return false;
				}
				if (data.workspaceId !== void 0) {
					let data0 = data.workspaceId;
					if (typeof data0 === "string") {
						if (func1(data0) > 128) {
							validate174.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate174.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate174.errors = [{
							instancePath: instancePath + "/workspaceId",
							schemaPath: "#/properties/workspaceId/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
				}
			}
		} else {
			validate174.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate174.errors = vErrors;
		return true;
	}
	validate174.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
}));
//#endregion
export default require_ipc_validators_input();
