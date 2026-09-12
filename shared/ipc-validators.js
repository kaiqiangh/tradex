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
	exports.AccountConnection = validate58;
	var schema42 = {
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
	var schema43 = {
		"type": "string",
		"enum": [
			"CONNECTING",
			"REVIEW_REQUIRED",
			"CONNECTED",
			"FAILED",
			"DISCONNECTED"
		]
	};
	var schema49 = {
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
	var func19 = Object.prototype.hasOwnProperty;
	var func1 = require_ucs2length().default;
	var schema44 = {
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
	var schema45 = {
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
	var schema46 = {
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
	var schema47 = {
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
	function validate26(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate26.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.remoteAccountId === void 0 && (missing0 = "remoteAccountId") || data.accountType === void 0 && (missing0 = "accountType") || data.balances === void 0 && (missing0 = "balances") || data.positions === void 0 && (missing0 = "positions") || data.openOrders === void 0 && (missing0 = "openOrders") || data.capabilities === void 0 && (missing0 = "capabilities") || data.limitations === void 0 && (missing0 = "limitations")) {
				validate26.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "accountType" || key0 === "balances" || key0 === "capabilities" || key0 === "currency" || key0 === "limitations" || key0 === "openOrders" || key0 === "positions" || key0 === "remoteAccountId")) {
					validate26.errors = [{
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
						validate26.errors = [{
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
										validate26.errors = [{
											instancePath: instancePath + "/balances/" + i0,
											schemaPath: "#/$defs/Balance/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "asset" || key1 === "available" || key1 === "inPies" || key1 === "locked" || key1 === "reserved" || key1 === "restrictedAvailable" || key1 === "total")) {
											validate26.errors = [{
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
												validate26.errors = [{
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
													validate26.errors = [{
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
														validate26.errors = [{
															instancePath: instancePath + "/balances/" + i0 + "/inPies",
															schemaPath: "#/$defs/Balance/properties/inPies/type",
															keyword: "type",
															params: { type: schema45.properties.inPies.type },
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
															validate26.errors = [{
																instancePath: instancePath + "/balances/" + i0 + "/locked",
																schemaPath: "#/$defs/Balance/properties/locked/type",
																keyword: "type",
																params: { type: schema45.properties.locked.type },
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
																validate26.errors = [{
																	instancePath: instancePath + "/balances/" + i0 + "/reserved",
																	schemaPath: "#/$defs/Balance/properties/reserved/type",
																	keyword: "type",
																	params: { type: schema45.properties.reserved.type },
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
																	validate26.errors = [{
																		instancePath: instancePath + "/balances/" + i0 + "/restrictedAvailable",
																		schemaPath: "#/$defs/Balance/properties/restrictedAvailable/type",
																		keyword: "type",
																		params: { type: schema45.properties.restrictedAvailable.type },
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
																		validate26.errors = [{
																			instancePath: instancePath + "/balances/" + i0 + "/total",
																			schemaPath: "#/$defs/Balance/properties/total/type",
																			keyword: "type",
																			params: { type: schema45.properties.total.type },
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
									validate26.errors = [{
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
							validate26.errors = [{
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
									validate26.errors = [{
										instancePath: instancePath + "/capabilities/" + i1,
										schemaPath: "#/properties/capabilities/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate26.errors = [{
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
									validate26.errors = [{
										instancePath: instancePath + "/currency",
										schemaPath: "#/properties/currency/type",
										keyword: "type",
										params: { type: schema44.properties.currency.type },
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
											validate26.errors = [{
												instancePath: instancePath + "/limitations/" + i2,
												schemaPath: "#/properties/limitations/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate26.errors = [{
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
														validate26.errors = [{
															instancePath: instancePath + "/openOrders/" + i3,
															schemaPath: "#/$defs/OpenOrder/required",
															keyword: "required",
															params: { missingProperty: missing2 },
															message: "must have required property '" + missing2 + "'"
														}];
														return false;
													} else {
														for (const key2 in data16) if (!func19.call(schema46.properties, key2)) {
															validate26.errors = [{
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
																validate26.errors = [{
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
																	validate26.errors = [{
																		instancePath: instancePath + "/openOrders/" + i3 + "/currency",
																		schemaPath: "#/$defs/OpenOrder/properties/currency/type",
																		keyword: "type",
																		params: { type: schema46.properties.currency.type },
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
																		validate26.errors = [{
																			instancePath: instancePath + "/openOrders/" + i3 + "/filledQuantity",
																			schemaPath: "#/$defs/OpenOrder/properties/filledQuantity/type",
																			keyword: "type",
																			params: { type: schema46.properties.filledQuantity.type },
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
																			validate26.errors = [{
																				instancePath: instancePath + "/openOrders/" + i3 + "/filledValue",
																				schemaPath: "#/$defs/OpenOrder/properties/filledValue/type",
																				keyword: "type",
																				params: { type: schema46.properties.filledValue.type },
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
																				validate26.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/type",
																					keyword: "type",
																					params: { type: schema46.properties.kind.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			if (!(data21 === "NORMAL" || data21 === "TPSL" || data21 === "PLAN" || data21 === null)) {
																				validate26.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/enum",
																					keyword: "enum",
																					params: { allowedValues: schema46.properties.kind.enum },
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
																					validate26.errors = [{
																						instancePath: instancePath + "/openOrders/" + i3 + "/limitPrice",
																						schemaPath: "#/$defs/OpenOrder/properties/limitPrice/type",
																						keyword: "type",
																						params: { type: schema46.properties.limitPrice.type },
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
																						validate26.errors = [{
																							instancePath: instancePath + "/openOrders/" + i3 + "/notional",
																							schemaPath: "#/$defs/OpenOrder/properties/notional/type",
																							keyword: "type",
																							params: { type: schema46.properties.notional.type },
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
																							validate26.errors = [{
																								instancePath: instancePath + "/openOrders/" + i3 + "/quantity",
																								schemaPath: "#/$defs/OpenOrder/properties/quantity/type",
																								keyword: "type",
																								params: { type: schema46.properties.quantity.type },
																								message: "must be string,null"
																							}];
																							return false;
																						}
																						var valid8 = true;
																					} else var valid8 = true;
																					if (valid8) {
																						if (data16.side !== void 0) {
																							if (typeof data16.side !== "string") {
																								validate26.errors = [{
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
																									validate26.errors = [{
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
																										validate26.errors = [{
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
																											validate26.errors = [{
																												instancePath: instancePath + "/openOrders/" + i3 + "/triggerPrice",
																												schemaPath: "#/$defs/OpenOrder/properties/triggerPrice/type",
																												keyword: "type",
																												params: { type: schema46.properties.triggerPrice.type },
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
													validate26.errors = [{
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
											validate26.errors = [{
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
															validate26.errors = [{
																instancePath: instancePath + "/positions/" + i4,
																schemaPath: "#/$defs/Position/required",
																keyword: "required",
																params: { missingProperty: missing3 },
																message: "must have required property '" + missing3 + "'"
															}];
															return false;
														} else {
															for (const key3 in data30) if (!(key3 === "averageEntryPrice" || key3 === "instrumentCurrency" || key3 === "marketValue" || key3 === "marketValueCurrency" || key3 === "quantity" || key3 === "symbol")) {
																validate26.errors = [{
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
																	validate26.errors = [{
																		instancePath: instancePath + "/positions/" + i4 + "/averageEntryPrice",
																		schemaPath: "#/$defs/Position/properties/averageEntryPrice/type",
																		keyword: "type",
																		params: { type: schema47.properties.averageEntryPrice.type },
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
																		validate26.errors = [{
																			instancePath: instancePath + "/positions/" + i4 + "/instrumentCurrency",
																			schemaPath: "#/$defs/Position/properties/instrumentCurrency/type",
																			keyword: "type",
																			params: { type: schema47.properties.instrumentCurrency.type },
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
																			validate26.errors = [{
																				instancePath: instancePath + "/positions/" + i4 + "/marketValue",
																				schemaPath: "#/$defs/Position/properties/marketValue/type",
																				keyword: "type",
																				params: { type: schema47.properties.marketValue.type },
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
																				validate26.errors = [{
																					instancePath: instancePath + "/positions/" + i4 + "/marketValueCurrency",
																					schemaPath: "#/$defs/Position/properties/marketValueCurrency/type",
																					keyword: "type",
																					params: { type: schema47.properties.marketValueCurrency.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			var valid11 = true;
																		} else var valid11 = true;
																		if (valid11) {
																			if (data30.quantity !== void 0) {
																				if (typeof data30.quantity !== "string") {
																					validate26.errors = [{
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
																						validate26.errors = [{
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
														validate26.errors = [{
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
												validate26.errors = [{
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
													validate26.errors = [{
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
	function validate58(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate58.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.connectionId === void 0 && (missing0 = "connectionId") || data.workspaceId === void 0 && (missing0 = "workspaceId") || data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment") || data.label === void 0 && (missing0 = "label") || data.createdAt === void 0 && (missing0 = "createdAt") || data.updatedAt === void 0 && (missing0 = "updatedAt") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.connectionState === void 0 && (missing0 = "connectionState") || data.health === void 0 && (missing0 = "health") || data.permissions === void 0 && (missing0 = "permissions")) {
					validate58.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func19.call(schema42.properties, key0)) {
						validate58.errors = [{
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
										validate58.errors = [{
											instancePath: instancePath + "/connectionId",
											schemaPath: "#/properties/connectionId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate58.errors = [{
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
									validate58.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CONNECTING" || data1 === "REVIEW_REQUIRED" || data1 === "CONNECTED" || data1 === "FAILED" || data1 === "DISCONNECTED")) {
									validate58.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/enum",
										keyword: "enum",
										params: { allowedValues: schema43.enum },
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
										validate58.errors = [{
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
										if (!validate26(data3, {
											instancePath: instancePath + "/data",
											parentData: data,
											parentDataProperty: "data",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate26.errors : vErrors.concat(validate26.errors);
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
											validate58.errors = vErrors;
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
												validate58.errors = [{
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
															validate58.errors = [{
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
																validate58.errors = [{
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
																		validate58.errors = [{
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
																			validate58.errors = [{
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
																				validate58.errors = [{
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
																					validate58.errors = [{
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
																						validate58.errors = [{
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
																							validate58.errors = [{
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
																								validate58.errors = [{
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
																									validate58.errors = [{
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
														validate58.errors = [{
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
														validate58.errors = [{
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
															validate58.errors = [{
																instancePath: instancePath + "/lastSuccessfulSync",
																schemaPath: "#/properties/lastSuccessfulSync/type",
																keyword: "type",
																params: { type: schema42.properties.lastSuccessfulSync.type },
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
																		validate58.errors = [{
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
																			validate58.errors = [{
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
																					validate58.errors = [{
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
																									validate58.errors = [{
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
																							validate58.errors = [{
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
																										validate58.errors = [{
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
																								validate58.errors = [{
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
																								validate58.errors = [{
																									instancePath: instancePath + "/permissions/ipAllowList",
																									schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
																									keyword: "type",
																									params: { type: schema49.properties.ipAllowList.type },
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
																											validate58.errors = [{
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
																									validate58.errors = [{
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
																										validate58.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(data25 === "VERIFIED" || data25 === "UNVERIFIED")) {
																										validate58.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
																											keyword: "enum",
																											params: { allowedValues: schema49.properties.scope.enum },
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
																														validate58.errors = [{
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
																												validate58.errors = [{
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
																	validate58.errors = [{
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
																	validate58.errors = [{
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
																				validate58.errors = [{
																					instancePath: instancePath + "/stateVersion",
																					schemaPath: "#/properties/stateVersion/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate58.errors = [{
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
																			validate58.errors = [{
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
																						validate58.errors = [{
																							instancePath: instancePath + "/workspaceId",
																							schemaPath: "#/properties/workspaceId/minLength",
																							keyword: "minLength",
																							params: { limit: 1 },
																							message: "must NOT have fewer than 1 characters"
																						}];
																						return false;
																					}
																				} else {
																					validate58.errors = [{
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
				validate58.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate58.errors = vErrors;
		return errors === 0;
	}
	validate58.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountData = validate60;
	function validate60(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate60.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.remoteAccountId === void 0 && (missing0 = "remoteAccountId") || data.accountType === void 0 && (missing0 = "accountType") || data.balances === void 0 && (missing0 = "balances") || data.positions === void 0 && (missing0 = "positions") || data.openOrders === void 0 && (missing0 = "openOrders") || data.capabilities === void 0 && (missing0 = "capabilities") || data.limitations === void 0 && (missing0 = "limitations")) {
				validate60.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "accountType" || key0 === "balances" || key0 === "capabilities" || key0 === "currency" || key0 === "limitations" || key0 === "openOrders" || key0 === "positions" || key0 === "remoteAccountId")) {
					validate60.errors = [{
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
						validate60.errors = [{
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
										validate60.errors = [{
											instancePath: instancePath + "/balances/" + i0,
											schemaPath: "#/$defs/Balance/required",
											keyword: "required",
											params: { missingProperty: missing1 },
											message: "must have required property '" + missing1 + "'"
										}];
										return false;
									} else {
										for (const key1 in data2) if (!(key1 === "asset" || key1 === "available" || key1 === "inPies" || key1 === "locked" || key1 === "reserved" || key1 === "restrictedAvailable" || key1 === "total")) {
											validate60.errors = [{
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
												validate60.errors = [{
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
													validate60.errors = [{
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
														validate60.errors = [{
															instancePath: instancePath + "/balances/" + i0 + "/inPies",
															schemaPath: "#/$defs/Balance/properties/inPies/type",
															keyword: "type",
															params: { type: schema45.properties.inPies.type },
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
															validate60.errors = [{
																instancePath: instancePath + "/balances/" + i0 + "/locked",
																schemaPath: "#/$defs/Balance/properties/locked/type",
																keyword: "type",
																params: { type: schema45.properties.locked.type },
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
																validate60.errors = [{
																	instancePath: instancePath + "/balances/" + i0 + "/reserved",
																	schemaPath: "#/$defs/Balance/properties/reserved/type",
																	keyword: "type",
																	params: { type: schema45.properties.reserved.type },
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
																	validate60.errors = [{
																		instancePath: instancePath + "/balances/" + i0 + "/restrictedAvailable",
																		schemaPath: "#/$defs/Balance/properties/restrictedAvailable/type",
																		keyword: "type",
																		params: { type: schema45.properties.restrictedAvailable.type },
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
																		validate60.errors = [{
																			instancePath: instancePath + "/balances/" + i0 + "/total",
																			schemaPath: "#/$defs/Balance/properties/total/type",
																			keyword: "type",
																			params: { type: schema45.properties.total.type },
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
									validate60.errors = [{
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
							validate60.errors = [{
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
									validate60.errors = [{
										instancePath: instancePath + "/capabilities/" + i1,
										schemaPath: "#/properties/capabilities/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate60.errors = [{
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
									validate60.errors = [{
										instancePath: instancePath + "/currency",
										schemaPath: "#/properties/currency/type",
										keyword: "type",
										params: { type: schema44.properties.currency.type },
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
											validate60.errors = [{
												instancePath: instancePath + "/limitations/" + i2,
												schemaPath: "#/properties/limitations/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate60.errors = [{
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
														validate60.errors = [{
															instancePath: instancePath + "/openOrders/" + i3,
															schemaPath: "#/$defs/OpenOrder/required",
															keyword: "required",
															params: { missingProperty: missing2 },
															message: "must have required property '" + missing2 + "'"
														}];
														return false;
													} else {
														for (const key2 in data16) if (!func19.call(schema46.properties, key2)) {
															validate60.errors = [{
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
																validate60.errors = [{
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
																	validate60.errors = [{
																		instancePath: instancePath + "/openOrders/" + i3 + "/currency",
																		schemaPath: "#/$defs/OpenOrder/properties/currency/type",
																		keyword: "type",
																		params: { type: schema46.properties.currency.type },
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
																		validate60.errors = [{
																			instancePath: instancePath + "/openOrders/" + i3 + "/filledQuantity",
																			schemaPath: "#/$defs/OpenOrder/properties/filledQuantity/type",
																			keyword: "type",
																			params: { type: schema46.properties.filledQuantity.type },
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
																			validate60.errors = [{
																				instancePath: instancePath + "/openOrders/" + i3 + "/filledValue",
																				schemaPath: "#/$defs/OpenOrder/properties/filledValue/type",
																				keyword: "type",
																				params: { type: schema46.properties.filledValue.type },
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
																				validate60.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/type",
																					keyword: "type",
																					params: { type: schema46.properties.kind.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			if (!(data21 === "NORMAL" || data21 === "TPSL" || data21 === "PLAN" || data21 === null)) {
																				validate60.errors = [{
																					instancePath: instancePath + "/openOrders/" + i3 + "/kind",
																					schemaPath: "#/$defs/OpenOrder/properties/kind/enum",
																					keyword: "enum",
																					params: { allowedValues: schema46.properties.kind.enum },
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
																					validate60.errors = [{
																						instancePath: instancePath + "/openOrders/" + i3 + "/limitPrice",
																						schemaPath: "#/$defs/OpenOrder/properties/limitPrice/type",
																						keyword: "type",
																						params: { type: schema46.properties.limitPrice.type },
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
																						validate60.errors = [{
																							instancePath: instancePath + "/openOrders/" + i3 + "/notional",
																							schemaPath: "#/$defs/OpenOrder/properties/notional/type",
																							keyword: "type",
																							params: { type: schema46.properties.notional.type },
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
																							validate60.errors = [{
																								instancePath: instancePath + "/openOrders/" + i3 + "/quantity",
																								schemaPath: "#/$defs/OpenOrder/properties/quantity/type",
																								keyword: "type",
																								params: { type: schema46.properties.quantity.type },
																								message: "must be string,null"
																							}];
																							return false;
																						}
																						var valid8 = true;
																					} else var valid8 = true;
																					if (valid8) {
																						if (data16.side !== void 0) {
																							if (typeof data16.side !== "string") {
																								validate60.errors = [{
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
																									validate60.errors = [{
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
																										validate60.errors = [{
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
																											validate60.errors = [{
																												instancePath: instancePath + "/openOrders/" + i3 + "/triggerPrice",
																												schemaPath: "#/$defs/OpenOrder/properties/triggerPrice/type",
																												keyword: "type",
																												params: { type: schema46.properties.triggerPrice.type },
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
													validate60.errors = [{
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
											validate60.errors = [{
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
															validate60.errors = [{
																instancePath: instancePath + "/positions/" + i4,
																schemaPath: "#/$defs/Position/required",
																keyword: "required",
																params: { missingProperty: missing3 },
																message: "must have required property '" + missing3 + "'"
															}];
															return false;
														} else {
															for (const key3 in data30) if (!(key3 === "averageEntryPrice" || key3 === "instrumentCurrency" || key3 === "marketValue" || key3 === "marketValueCurrency" || key3 === "quantity" || key3 === "symbol")) {
																validate60.errors = [{
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
																	validate60.errors = [{
																		instancePath: instancePath + "/positions/" + i4 + "/averageEntryPrice",
																		schemaPath: "#/$defs/Position/properties/averageEntryPrice/type",
																		keyword: "type",
																		params: { type: schema47.properties.averageEntryPrice.type },
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
																		validate60.errors = [{
																			instancePath: instancePath + "/positions/" + i4 + "/instrumentCurrency",
																			schemaPath: "#/$defs/Position/properties/instrumentCurrency/type",
																			keyword: "type",
																			params: { type: schema47.properties.instrumentCurrency.type },
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
																			validate60.errors = [{
																				instancePath: instancePath + "/positions/" + i4 + "/marketValue",
																				schemaPath: "#/$defs/Position/properties/marketValue/type",
																				keyword: "type",
																				params: { type: schema47.properties.marketValue.type },
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
																				validate60.errors = [{
																					instancePath: instancePath + "/positions/" + i4 + "/marketValueCurrency",
																					schemaPath: "#/$defs/Position/properties/marketValueCurrency/type",
																					keyword: "type",
																					params: { type: schema47.properties.marketValueCurrency.type },
																					message: "must be string,null"
																				}];
																				return false;
																			}
																			var valid11 = true;
																		} else var valid11 = true;
																		if (valid11) {
																			if (data30.quantity !== void 0) {
																				if (typeof data30.quantity !== "string") {
																					validate60.errors = [{
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
																						validate60.errors = [{
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
														validate60.errors = [{
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
												validate60.errors = [{
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
													validate60.errors = [{
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
			validate60.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate60.errors = vErrors;
		return true;
	}
	validate60.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountHealth = validate61;
	function validate61(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate61.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.connection === void 0 && (missing0 = "connection") || data.authentication === void 0 && (missing0 = "authentication") || data.credential === void 0 && (missing0 = "credential") || data.privateStream === void 0 && (missing0 = "privateStream") || data.reconciliation === void 0 && (missing0 = "reconciliation") || data.executionEligibility === void 0 && (missing0 = "executionEligibility") || data.arming === void 0 && (missing0 = "arming") || data.reason === void 0 && (missing0 = "reason")) {
				validate61.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "arming" || key0 === "authentication" || key0 === "connection" || key0 === "credential" || key0 === "executionEligibility" || key0 === "privateStream" || key0 === "reason" || key0 === "reconciliation")) {
					validate61.errors = [{
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
						validate61.errors = [{
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
							validate61.errors = [{
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
								validate61.errors = [{
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
									validate61.errors = [{
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
										validate61.errors = [{
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
											validate61.errors = [{
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
												validate61.errors = [{
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
													validate61.errors = [{
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
	exports.AccountMutation = validate62;
	function validate62(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate62.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.connectionId === void 0 && (missing0 = "connectionId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion")) {
				validate62.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "connectionId" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate62.errors = [{
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
							validate62.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate62.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate62.errors = [{
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
								validate62.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate62.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate62.errors = [{
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
									validate62.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate62.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate62.errors = [{
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
			validate62.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate62.errors = vErrors;
		return true;
	}
	validate62.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.AccountQuery = validate63;
	function validate63(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate63.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.connectionId === void 0 && (missing0 = "connectionId")) {
				validate63.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "connectionId" || key0 === "workspaceId")) {
					validate63.errors = [{
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
							validate63.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate63.errors = [{
								instancePath: instancePath + "/connectionId",
								schemaPath: "#/properties/connectionId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate63.errors = [{
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
								validate63.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate63.errors = [{
									instancePath: instancePath + "/workspaceId",
									schemaPath: "#/properties/workspaceId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate63.errors = [{
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
			validate63.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate63.errors = vErrors;
		return true;
	}
	validate63.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Accounts = validate64;
	function validate25(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate25.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.connectionId === void 0 && (missing0 = "connectionId") || data.workspaceId === void 0 && (missing0 = "workspaceId") || data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment") || data.label === void 0 && (missing0 = "label") || data.createdAt === void 0 && (missing0 = "createdAt") || data.updatedAt === void 0 && (missing0 = "updatedAt") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.connectionState === void 0 && (missing0 = "connectionState") || data.health === void 0 && (missing0 = "health") || data.permissions === void 0 && (missing0 = "permissions")) {
					validate25.errors = [{
						instancePath,
						schemaPath: "#/required",
						keyword: "required",
						params: { missingProperty: missing0 },
						message: "must have required property '" + missing0 + "'"
					}];
					return false;
				} else {
					const _errs1 = errors;
					for (const key0 in data) if (!func19.call(schema42.properties, key0)) {
						validate25.errors = [{
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
										validate25.errors = [{
											instancePath: instancePath + "/connectionId",
											schemaPath: "#/properties/connectionId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate25.errors = [{
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
									validate25.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "CONNECTING" || data1 === "REVIEW_REQUIRED" || data1 === "CONNECTED" || data1 === "FAILED" || data1 === "DISCONNECTED")) {
									validate25.errors = [{
										instancePath: instancePath + "/connectionState",
										schemaPath: "#/$defs/ConnectionState/enum",
										keyword: "enum",
										params: { allowedValues: schema43.enum },
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
										validate25.errors = [{
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
										if (!validate26(data3, {
											instancePath: instancePath + "/data",
											parentData: data,
											parentDataProperty: "data",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate26.errors : vErrors.concat(validate26.errors);
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
											validate25.errors = vErrors;
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
												validate25.errors = [{
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
															validate25.errors = [{
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
																validate25.errors = [{
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
																		validate25.errors = [{
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
																			validate25.errors = [{
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
																				validate25.errors = [{
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
																					validate25.errors = [{
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
																						validate25.errors = [{
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
																							validate25.errors = [{
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
																								validate25.errors = [{
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
																									validate25.errors = [{
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
														validate25.errors = [{
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
														validate25.errors = [{
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
															validate25.errors = [{
																instancePath: instancePath + "/lastSuccessfulSync",
																schemaPath: "#/properties/lastSuccessfulSync/type",
																keyword: "type",
																params: { type: schema42.properties.lastSuccessfulSync.type },
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
																		validate25.errors = [{
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
																			validate25.errors = [{
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
																					validate25.errors = [{
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
																									validate25.errors = [{
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
																							validate25.errors = [{
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
																										validate25.errors = [{
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
																								validate25.errors = [{
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
																								validate25.errors = [{
																									instancePath: instancePath + "/permissions/ipAllowList",
																									schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
																									keyword: "type",
																									params: { type: schema49.properties.ipAllowList.type },
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
																											validate25.errors = [{
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
																									validate25.errors = [{
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
																										validate25.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/type",
																											keyword: "type",
																											params: { type: "string" },
																											message: "must be string"
																										}];
																										return false;
																									}
																									if (!(data25 === "VERIFIED" || data25 === "UNVERIFIED")) {
																										validate25.errors = [{
																											instancePath: instancePath + "/permissions/scope",
																											schemaPath: "#/$defs/PermissionReview/properties/scope/enum",
																											keyword: "enum",
																											params: { allowedValues: schema49.properties.scope.enum },
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
																														validate25.errors = [{
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
																												validate25.errors = [{
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
																	validate25.errors = [{
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
																	validate25.errors = [{
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
																	}
																	var valid0 = _errs68 === errors;
																} else var valid0 = true;
																if (valid0) {
																	if (data.updatedAt !== void 0) {
																		const _errs70 = errors;
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
																		var valid0 = _errs70 === errors;
																	} else var valid0 = true;
																	if (valid0) {
																		if (data.workspaceId !== void 0) {
																			let data31 = data.workspaceId;
																			const _errs72 = errors;
																			if (errors === _errs72) {
																				if (typeof data31 === "string") {
																					if (func1(data31) < 1) {
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
				validate25.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate25.errors = vErrors;
		return errors === 0;
	}
	validate25.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate64(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate64.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.accounts === void 0 && (missing0 = "accounts")) {
					validate64.errors = [{
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
						validate64.errors = [{
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
										if (!validate25(data0[i0], {
											instancePath: instancePath + "/accounts/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate64.errors = [{
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
				validate64.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate64.errors = vErrors;
		return errors === 0;
	}
	validate64.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Aggregate = validate66;
	function validate66(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate66.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId")) {
				validate66.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType")) {
					validate66.errors = [{
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
							validate66.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate66.errors = [{
								instancePath: instancePath + "/aggregateId",
								schemaPath: "#/properties/aggregateId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate66.errors = [{
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
								validate66.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/maxLength",
									keyword: "maxLength",
									params: { limit: 64 },
									message: "must NOT have more than 64 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate66.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate66.errors = [{
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
			validate66.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate66.errors = vErrors;
		return true;
	}
	validate66.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Balance = validate67;
	function validate67(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate67.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.asset === void 0 && (missing0 = "asset") || data.available === void 0 && (missing0 = "available")) {
				validate67.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "asset" || key0 === "available" || key0 === "inPies" || key0 === "locked" || key0 === "reserved" || key0 === "restrictedAvailable" || key0 === "total")) {
					validate67.errors = [{
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
						validate67.errors = [{
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
							validate67.errors = [{
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
								validate67.errors = [{
									instancePath: instancePath + "/inPies",
									schemaPath: "#/properties/inPies/type",
									keyword: "type",
									params: { type: schema45.properties.inPies.type },
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
									validate67.errors = [{
										instancePath: instancePath + "/locked",
										schemaPath: "#/properties/locked/type",
										keyword: "type",
										params: { type: schema45.properties.locked.type },
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
										validate67.errors = [{
											instancePath: instancePath + "/reserved",
											schemaPath: "#/properties/reserved/type",
											keyword: "type",
											params: { type: schema45.properties.reserved.type },
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
											validate67.errors = [{
												instancePath: instancePath + "/restrictedAvailable",
												schemaPath: "#/properties/restrictedAvailable/type",
												keyword: "type",
												params: { type: schema45.properties.restrictedAvailable.type },
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
												validate67.errors = [{
													instancePath: instancePath + "/total",
													schemaPath: "#/properties/total/type",
													keyword: "type",
													params: { type: schema45.properties.total.type },
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
			validate67.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate67.errors = vErrors;
		return true;
	}
	validate67.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.CommandEnvelope = validate68;
	function validate68(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate68.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.command === void 0 && (missing0 = "command") || data.payload === void 0 && (missing0 = "payload")) {
				validate68.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "command" || key0 === "payload" || key0 === "requestId" || key0 === "schemaVersion")) {
					validate68.errors = [{
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
						validate68.errors = [{
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
								validate68.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate68.errors = [{
									instancePath: instancePath + "/requestId",
									schemaPath: "#/properties/requestId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate68.errors = [{
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
								validate68.errors = [{
									instancePath: instancePath + "/schemaVersion",
									schemaPath: "#/properties/schemaVersion/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								}];
								return false;
							}
							if (1 !== data2) {
								validate68.errors = [{
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
									validate68.errors = [{
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
			validate68.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate68.errors = vErrors;
		return true;
	}
	validate68.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Connect = validate69;
	function validate69(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate69.evaluated;
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
			validate69.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate69.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate69.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.ConnectionState = validate70;
	function validate70(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate70.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate70.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "CONNECTING" || data === "REVIEW_REQUIRED" || data === "CONNECTED" || data === "FAILED" || data === "DISCONNECTED")) {
			validate70.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema43.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate70.errors = vErrors;
		return true;
	}
	validate70.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.DomainEvent = validate71;
	var schema37 = {
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
					"model-gateway"
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
					"model.gateway.changed"
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
	var schema39 = {
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
	var schema40 = {
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
	function validate23(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate23.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.pinnedVersion === void 0 && (missing0 = "pinnedVersion") || data.endpoint === void 0 && (missing0 = "endpoint") || data.status === void 0 && (missing0 = "status") || data.desiredRunning === void 0 && (missing0 = "desiredRunning") || data.installed === void 0 && (missing0 = "installed") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.discoveredModelCount === void 0 && (missing0 = "discoveredModelCount") || data.restartAttempts === void 0 && (missing0 = "restartAttempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate23.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func19.call(schema39.properties, key0)) {
					validate23.errors = [{
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
						validate23.errors = [{
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
							validate23.errors = [{
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
								validate23.errors = [{
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
								validate23.errors = [{
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
								validate23.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if ("http://127.0.0.1:8317" !== data2) {
								validate23.errors = [{
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
									validate23.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema39.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.installed !== void 0) {
									if (typeof data.installed !== "boolean") {
										validate23.errors = [{
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
											validate23.errors = [{
												instancePath: instancePath + "/lastProbeAt",
												schemaPath: "#/properties/lastProbeAt/type",
												keyword: "type",
												params: { type: schema39.properties.lastProbeAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelAvailable !== void 0) {
											if (typeof data.modelAvailable !== "boolean") {
												validate23.errors = [{
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
													validate23.errors = [{
														instancePath: instancePath + "/nextRetryAt",
														schemaPath: "#/properties/nextRetryAt/type",
														keyword: "type",
														params: { type: schema39.properties.nextRetryAt.type },
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
														validate23.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if ("7.2.155" !== data8) {
														validate23.errors = [{
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
															validate23.errors = [{
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
																validate23.errors = [{
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
																validate23.errors = [{
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
																	validate23.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data10) < 1) {
																	validate23.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate23.errors = [{
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
																	validate23.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																if (!(data11 === "STOPPED" || data11 === "INSTALLING" || data11 === "STARTING" || data11 === "RUNNING" || data11 === "PORT_CONFLICT" || data11 === "UNAUTHORIZED" || data11 === "BACKOFF" || data11 === "FAILED" || data11 === "STOPPING")) {
																	validate23.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/enum",
																		keyword: "enum",
																		params: { allowedValues: schema40.enum },
																		message: "must be equal to one of the allowed values"
																	}];
																	return false;
																}
																var valid0 = true;
															} else var valid0 = true;
															if (valid0) {
																if (data.updatedAt !== void 0) {
																	if (typeof data.updatedAt !== "string") {
																		validate23.errors = [{
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
																				validate23.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/maxLength",
																					keyword: "maxLength",
																					params: { limit: 128 },
																					message: "must NOT have more than 128 characters"
																				}];
																				return false;
																			} else if (func1(data13) < 1) {
																				validate23.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate23.errors = [{
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
			validate23.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate23.errors = vErrors;
		return true;
	}
	validate23.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate22(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate22.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate23(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
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
					const _errs5 = errors;
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
					if (_errs5 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs6 = errors;
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
							var valid2 = _errs6 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs8 = errors;
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
								var valid2 = _errs8 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs10 = errors;
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
									var valid2 = _errs10 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs12 = errors;
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
										var valid2 = _errs12 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs14 = errors;
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
											var valid2 = _errs14 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs16 = errors;
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
												if (errors === _errs16) {
													if (typeof data5 == "number") {
														if (data5 > 3 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 3
																},
																message: "must be <= 3"
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
												var valid2 = _errs16 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													let data6 = data.workspaceId;
													const _errs18 = errors;
													if (errors === _errs18) {
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
													var valid2 = _errs18 === errors;
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
		var _valid0 = _errs2 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
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
		var _valid0 = _errs20 === errors;
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
			validate22.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate22.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate22.evaluated = {
		"dynamicProps": true,
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
				if (data.eventId === void 0 && (missing0 = "eventId") || data.eventType === void 0 && (missing0 = "eventType") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.occurredAt === void 0 && (missing0 = "occurredAt") || data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.sequence === void 0 && (missing0 = "sequence") || data.payload === void 0 && (missing0 = "payload")) {
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
					for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "eventId" || key0 === "eventType" || key0 === "occurredAt" || key0 === "payload" || key0 === "schemaVersion" || key0 === "sequence")) {
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
						if (data.aggregateId !== void 0) {
							let data0 = data.aggregateId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate71.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate71.errors = [{
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
									validate71.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway")) {
									validate71.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema37.properties.aggregateType.enum },
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
												validate71.errors = [{
													instancePath: instancePath + "/eventId",
													schemaPath: "#/properties/eventId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate71.errors = [{
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
											validate71.errors = [{
												instancePath: instancePath + "/eventType",
												schemaPath: "#/properties/eventType/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data3 === "workspace.opened" || data3 === "account.health.changed" || data3 === "model.gateway.changed")) {
											validate71.errors = [{
												instancePath: instancePath + "/eventType",
												schemaPath: "#/properties/eventType/enum",
												keyword: "enum",
												params: { allowedValues: schema37.properties.eventType.enum },
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
												validate71.errors = [{
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
												if (!validate22(data.payload, {
													instancePath: instancePath + "/payload",
													parentData: data,
													parentDataProperty: "payload",
													rootData,
													dynamicAnchors
												})) {
													vErrors = vErrors === null ? validate22.errors : vErrors.concat(validate22.errors);
													errors = vErrors.length;
												}
												var valid0 = _errs12 === errors;
											} else var valid0 = true;
											if (valid0) {
												if (data.schemaVersion !== void 0) {
													let data6 = data.schemaVersion;
													const _errs13 = errors;
													if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6))) {
														validate71.errors = [{
															instancePath: instancePath + "/schemaVersion",
															schemaPath: "#/properties/schemaVersion/type",
															keyword: "type",
															params: { type: "integer" },
															message: "must be integer"
														}];
														return false;
													}
													if (1 !== data6) {
														validate71.errors = [{
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
													var valid0 = _errs13 === errors;
												} else var valid0 = true;
												if (valid0) {
													if (data.sequence !== void 0) {
														let data7 = data.sequence;
														const _errs15 = errors;
														if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7))) {
															validate71.errors = [{
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
																	validate71.errors = [{
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
																	validate71.errors = [{
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
	exports.DomainProjection = validate73;
	function validate73(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate73.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate23(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
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
					const _errs5 = errors;
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
					if (_errs5 === errors) {
						if (data.baseCurrency !== void 0) {
							const _errs6 = errors;
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
							var valid2 = _errs6 === errors;
						} else var valid2 = true;
						if (valid2) {
							if (data.createdAt !== void 0) {
								const _errs8 = errors;
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
								var valid2 = _errs8 === errors;
							} else var valid2 = true;
							if (valid2) {
								if (data.lastOpenedAt !== void 0) {
									const _errs10 = errors;
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
									var valid2 = _errs10 === errors;
								} else var valid2 = true;
								if (valid2) {
									if (data.name !== void 0) {
										const _errs12 = errors;
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
										var valid2 = _errs12 === errors;
									} else var valid2 = true;
									if (valid2) {
										if (data.path !== void 0) {
											const _errs14 = errors;
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
											var valid2 = _errs14 === errors;
										} else var valid2 = true;
										if (valid2) {
											if (data.storageSchemaVersion !== void 0) {
												let data5 = data.storageSchemaVersion;
												const _errs16 = errors;
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
												if (errors === _errs16) {
													if (typeof data5 == "number") {
														if (data5 > 3 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 3
																},
																message: "must be <= 3"
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
												var valid2 = _errs16 === errors;
											} else var valid2 = true;
											if (valid2) {
												if (data.workspaceId !== void 0) {
													let data6 = data.workspaceId;
													const _errs18 = errors;
													if (errors === _errs18) {
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
													var valid2 = _errs18 === errors;
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
		var _valid0 = _errs2 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
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
		var _valid0 = _errs20 === errors;
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
			validate73.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate73.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate73.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.EmptyPayload = validate76;
	function validate76(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate76.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) for (const key0 in data) {
			validate76.errors = [{
				instancePath,
				schemaPath: "#/additionalProperties",
				keyword: "additionalProperties",
				params: { additionalProperty: key0 },
				message: "must NOT have additional properties"
			}];
			return false;
		}
		else {
			validate76.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate76.errors = vErrors;
		return true;
	}
	validate76.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.FailureEnvelope = validate77;
	function validate54(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate54.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate54.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate54.errors = [{
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
						validate54.errors = [{
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
							validate54.errors = [{
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
								validate54.errors = [{
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
									validate54.errors = [{
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
													validate54.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate54.errors = [{
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
																validate54.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																	schemaPath: "#/$defs/Remediation/properties/id/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																}];
																return false;
															}
														} else {
															validate54.errors = [{
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
																validate54.errors = [{
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
												validate54.errors = [{
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
										validate54.errors = [{
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
											validate54.errors = [{
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
	function validate77(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate77.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate77.errors = [{
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
						validate77.errors = [{
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
							if (!validate54(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate54.errors : vErrors.concat(validate54.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate77.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate77.errors = [{
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
												validate77.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate77.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate77.errors = [{
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
											validate77.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate77.errors = [{
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
													validate77.errors = [{
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
				validate77.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate77.errors = vErrors;
		return errors === 0;
	}
	validate77.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayAction = validate79;
	var schema51 = {
		"type": "string",
		"enum": [
			"LAUNCH",
			"PROBE",
			"RESTART",
			"STOP"
		]
	};
	function validate79(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate79.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate79.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "LAUNCH" || data === "PROBE" || data === "RESTART" || data === "STOP")) {
			validate79.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema51.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate79.errors = vErrors;
		return true;
	}
	validate79.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayMutation = validate80;
	function validate80(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate80.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.expectedStateVersion === void 0 && (missing0 = "expectedStateVersion") || data.action === void 0 && (missing0 = "action")) {
				validate80.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "action" || key0 === "expectedStateVersion" || key0 === "workspaceId")) {
					validate80.errors = [{
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
						validate80.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/GatewayAction/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						}];
						return false;
					}
					if (!(data0 === "LAUNCH" || data0 === "PROBE" || data0 === "RESTART" || data0 === "STOP")) {
						validate80.errors = [{
							instancePath: instancePath + "/action",
							schemaPath: "#/$defs/GatewayAction/enum",
							keyword: "enum",
							params: { allowedValues: schema51.enum },
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
								validate80.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/maxLength",
									keyword: "maxLength",
									params: { limit: 256 },
									message: "must NOT have more than 256 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate80.errors = [{
									instancePath: instancePath + "/expectedStateVersion",
									schemaPath: "#/properties/expectedStateVersion/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate80.errors = [{
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
									validate80.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/maxLength",
										keyword: "maxLength",
										params: { limit: 128 },
										message: "must NOT have more than 128 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate80.errors = [{
										instancePath: instancePath + "/workspaceId",
										schemaPath: "#/properties/workspaceId/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate80.errors = [{
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
			validate80.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate80.errors = vErrors;
		return true;
	}
	validate80.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayState = validate81;
	function validate81(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate81.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.stateVersion === void 0 && (missing0 = "stateVersion") || data.pinnedVersion === void 0 && (missing0 = "pinnedVersion") || data.endpoint === void 0 && (missing0 = "endpoint") || data.status === void 0 && (missing0 = "status") || data.desiredRunning === void 0 && (missing0 = "desiredRunning") || data.installed === void 0 && (missing0 = "installed") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.discoveredModelCount === void 0 && (missing0 = "discoveredModelCount") || data.restartAttempts === void 0 && (missing0 = "restartAttempts") || data.updatedAt === void 0 && (missing0 = "updatedAt")) {
				validate81.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func19.call(schema39.properties, key0)) {
					validate81.errors = [{
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
						validate81.errors = [{
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
							validate81.errors = [{
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
								validate81.errors = [{
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
								validate81.errors = [{
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
								validate81.errors = [{
									instancePath: instancePath + "/endpoint",
									schemaPath: "#/properties/endpoint/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if ("http://127.0.0.1:8317" !== data2) {
								validate81.errors = [{
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
									validate81.errors = [{
										instancePath: instancePath + "/errorCode",
										schemaPath: "#/properties/errorCode/type",
										keyword: "type",
										params: { type: schema39.properties.errorCode.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.installed !== void 0) {
									if (typeof data.installed !== "boolean") {
										validate81.errors = [{
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
											validate81.errors = [{
												instancePath: instancePath + "/lastProbeAt",
												schemaPath: "#/properties/lastProbeAt/type",
												keyword: "type",
												params: { type: schema39.properties.lastProbeAt.type },
												message: "must be string,null"
											}];
											return false;
										}
										var valid0 = true;
									} else var valid0 = true;
									if (valid0) {
										if (data.modelAvailable !== void 0) {
											if (typeof data.modelAvailable !== "boolean") {
												validate81.errors = [{
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
													validate81.errors = [{
														instancePath: instancePath + "/nextRetryAt",
														schemaPath: "#/properties/nextRetryAt/type",
														keyword: "type",
														params: { type: schema39.properties.nextRetryAt.type },
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
														validate81.errors = [{
															instancePath: instancePath + "/pinnedVersion",
															schemaPath: "#/properties/pinnedVersion/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}];
														return false;
													}
													if ("7.2.155" !== data8) {
														validate81.errors = [{
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
															validate81.errors = [{
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
																validate81.errors = [{
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
																validate81.errors = [{
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
																	validate81.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/maxLength",
																		keyword: "maxLength",
																		params: { limit: 256 },
																		message: "must NOT have more than 256 characters"
																	}];
																	return false;
																} else if (func1(data10) < 1) {
																	validate81.errors = [{
																		instancePath: instancePath + "/stateVersion",
																		schemaPath: "#/properties/stateVersion/minLength",
																		keyword: "minLength",
																		params: { limit: 1 },
																		message: "must NOT have fewer than 1 characters"
																	}];
																	return false;
																}
															} else {
																validate81.errors = [{
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
																	validate81.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	}];
																	return false;
																}
																if (!(data11 === "STOPPED" || data11 === "INSTALLING" || data11 === "STARTING" || data11 === "RUNNING" || data11 === "PORT_CONFLICT" || data11 === "UNAUTHORIZED" || data11 === "BACKOFF" || data11 === "FAILED" || data11 === "STOPPING")) {
																	validate81.errors = [{
																		instancePath: instancePath + "/status",
																		schemaPath: "#/$defs/GatewayStatus/enum",
																		keyword: "enum",
																		params: { allowedValues: schema40.enum },
																		message: "must be equal to one of the allowed values"
																	}];
																	return false;
																}
																var valid0 = true;
															} else var valid0 = true;
															if (valid0) {
																if (data.updatedAt !== void 0) {
																	if (typeof data.updatedAt !== "string") {
																		validate81.errors = [{
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
																				validate81.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/maxLength",
																					keyword: "maxLength",
																					params: { limit: 128 },
																					message: "must NOT have more than 128 characters"
																				}];
																				return false;
																			} else if (func1(data13) < 1) {
																				validate81.errors = [{
																					instancePath: instancePath + "/workspaceId",
																					schemaPath: "#/properties/workspaceId/minLength",
																					keyword: "minLength",
																					params: { limit: 1 },
																					message: "must NOT have fewer than 1 characters"
																				}];
																				return false;
																			}
																		} else {
																			validate81.errors = [{
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
			validate81.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate81.errors = vErrors;
		return true;
	}
	validate81.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.GatewayStatus = validate82;
	function validate82(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate82.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (typeof data !== "string") {
			validate82.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			}];
			return false;
		}
		if (!(data === "STOPPED" || data === "INSTALLING" || data === "STARTING" || data === "RUNNING" || data === "PORT_CONFLICT" || data === "UNAUTHORIZED" || data === "BACKOFF" || data === "FAILED" || data === "STOPPING")) {
			validate82.errors = [{
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema40.enum },
				message: "must be equal to one of the allowed values"
			}];
			return false;
		}
		validate82.errors = vErrors;
		return true;
	}
	validate82.evaluated = {
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.OpenOrder = validate83;
	function validate83(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate83.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.brokerOrderId === void 0 && (missing0 = "brokerOrderId") || data.symbol === void 0 && (missing0 = "symbol") || data.side === void 0 && (missing0 = "side") || data.status === void 0 && (missing0 = "status")) {
				validate83.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func19.call(schema46.properties, key0)) {
					validate83.errors = [{
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
						validate83.errors = [{
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
							validate83.errors = [{
								instancePath: instancePath + "/currency",
								schemaPath: "#/properties/currency/type",
								keyword: "type",
								params: { type: schema46.properties.currency.type },
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
								validate83.errors = [{
									instancePath: instancePath + "/filledQuantity",
									schemaPath: "#/properties/filledQuantity/type",
									keyword: "type",
									params: { type: schema46.properties.filledQuantity.type },
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
									validate83.errors = [{
										instancePath: instancePath + "/filledValue",
										schemaPath: "#/properties/filledValue/type",
										keyword: "type",
										params: { type: schema46.properties.filledValue.type },
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
										validate83.errors = [{
											instancePath: instancePath + "/kind",
											schemaPath: "#/properties/kind/type",
											keyword: "type",
											params: { type: schema46.properties.kind.type },
											message: "must be string,null"
										}];
										return false;
									}
									if (!(data4 === "NORMAL" || data4 === "TPSL" || data4 === "PLAN" || data4 === null)) {
										validate83.errors = [{
											instancePath: instancePath + "/kind",
											schemaPath: "#/properties/kind/enum",
											keyword: "enum",
											params: { allowedValues: schema46.properties.kind.enum },
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
											validate83.errors = [{
												instancePath: instancePath + "/limitPrice",
												schemaPath: "#/properties/limitPrice/type",
												keyword: "type",
												params: { type: schema46.properties.limitPrice.type },
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
												validate83.errors = [{
													instancePath: instancePath + "/notional",
													schemaPath: "#/properties/notional/type",
													keyword: "type",
													params: { type: schema46.properties.notional.type },
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
													validate83.errors = [{
														instancePath: instancePath + "/quantity",
														schemaPath: "#/properties/quantity/type",
														keyword: "type",
														params: { type: schema46.properties.quantity.type },
														message: "must be string,null"
													}];
													return false;
												}
												var valid0 = true;
											} else var valid0 = true;
											if (valid0) {
												if (data.side !== void 0) {
													if (typeof data.side !== "string") {
														validate83.errors = [{
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
															validate83.errors = [{
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
																validate83.errors = [{
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
																	validate83.errors = [{
																		instancePath: instancePath + "/triggerPrice",
																		schemaPath: "#/properties/triggerPrice/type",
																		keyword: "type",
																		params: { type: schema46.properties.triggerPrice.type },
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
			validate83.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate83.errors = vErrors;
		return true;
	}
	validate83.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.OpenWorkspace = validate84;
	var pattern4 = /* @__PURE__ */ new RegExp("^[A-Z]{3}$", "u");
	function validate84(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate84.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "name" || key0 === "path")) {
				validate84.errors = [{
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
						validate84.errors = [{
							instancePath: instancePath + "/baseCurrency",
							schemaPath: "#/properties/baseCurrency/pattern",
							keyword: "pattern",
							params: { pattern: "^[A-Z]{3}$" },
							message: "must match pattern \"^[A-Z]{3}$\""
						}];
						return false;
					}
				} else {
					validate84.errors = [{
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
							validate84.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/maxLength",
								keyword: "maxLength",
								params: { limit: 120 },
								message: "must NOT have more than 120 characters"
							}];
							return false;
						} else if (func1(data1) < 1) {
							validate84.errors = [{
								instancePath: instancePath + "/name",
								schemaPath: "#/properties/name/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate84.errors = [{
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
							validate84.errors = [{
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
	exports.PermissionReview = validate85;
	function validate85(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate85.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.scope === void 0 && (missing0 = "scope") || data.detected === void 0 && (missing0 = "detected") || data.forbidden === void 0 && (missing0 = "forbidden") || data.unsupported === void 0 && (missing0 = "unsupported") || data.acknowledged === void 0 && (missing0 = "acknowledged") || data.ipAllowListStatus === void 0 && (missing0 = "ipAllowListStatus")) {
				validate85.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "acknowledged" || key0 === "detected" || key0 === "forbidden" || key0 === "ipAllowList" || key0 === "ipAllowListStatus" || key0 === "scope" || key0 === "unsupported")) {
					validate85.errors = [{
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
						validate85.errors = [{
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
								validate85.errors = [{
									instancePath: instancePath + "/detected/" + i0,
									schemaPath: "#/properties/detected/items/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
						} else {
							validate85.errors = [{
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
									validate85.errors = [{
										instancePath: instancePath + "/forbidden/" + i1,
										schemaPath: "#/properties/forbidden/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
							} else {
								validate85.errors = [{
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
									validate85.errors = [{
										instancePath: instancePath + "/ipAllowList",
										schemaPath: "#/properties/ipAllowList/type",
										keyword: "type",
										params: { type: schema49.properties.ipAllowList.type },
										message: "must be array,null"
									}];
									return false;
								}
								if (Array.isArray(data5)) {
									const len2 = data5.length;
									for (let i2 = 0; i2 < len2; i2++) if (typeof data5[i2] !== "string") {
										validate85.errors = [{
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
										validate85.errors = [{
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
											validate85.errors = [{
												instancePath: instancePath + "/scope",
												schemaPath: "#/properties/scope/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
										if (!(data8 === "VERIFIED" || data8 === "UNVERIFIED")) {
											validate85.errors = [{
												instancePath: instancePath + "/scope",
												schemaPath: "#/properties/scope/enum",
												keyword: "enum",
												params: { allowedValues: schema49.properties.scope.enum },
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
													validate85.errors = [{
														instancePath: instancePath + "/unsupported/" + i3,
														schemaPath: "#/properties/unsupported/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate85.errors = [{
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
	exports.Position = validate86;
	function validate86(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate86.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.symbol === void 0 && (missing0 = "symbol") || data.quantity === void 0 && (missing0 = "quantity")) {
				validate86.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "averageEntryPrice" || key0 === "instrumentCurrency" || key0 === "marketValue" || key0 === "marketValueCurrency" || key0 === "quantity" || key0 === "symbol")) {
					validate86.errors = [{
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
						validate86.errors = [{
							instancePath: instancePath + "/averageEntryPrice",
							schemaPath: "#/properties/averageEntryPrice/type",
							keyword: "type",
							params: { type: schema47.properties.averageEntryPrice.type },
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
							validate86.errors = [{
								instancePath: instancePath + "/instrumentCurrency",
								schemaPath: "#/properties/instrumentCurrency/type",
								keyword: "type",
								params: { type: schema47.properties.instrumentCurrency.type },
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
								validate86.errors = [{
									instancePath: instancePath + "/marketValue",
									schemaPath: "#/properties/marketValue/type",
									keyword: "type",
									params: { type: schema47.properties.marketValue.type },
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
									validate86.errors = [{
										instancePath: instancePath + "/marketValueCurrency",
										schemaPath: "#/properties/marketValueCurrency/type",
										keyword: "type",
										params: { type: schema47.properties.marketValueCurrency.type },
										message: "must be string,null"
									}];
									return false;
								}
								var valid0 = true;
							} else var valid0 = true;
							if (valid0) {
								if (data.quantity !== void 0) {
									if (typeof data.quantity !== "string") {
										validate86.errors = [{
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
											validate86.errors = [{
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
	exports.ProviderCatalog = validate87;
	var schema63 = {
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
	function validate43(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate43.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.displayName === void 0 && (missing0 = "displayName") || data.environment === void 0 && (missing0 = "environment") || data.available === void 0 && (missing0 = "available") || data.helpText === void 0 && (missing0 = "helpText") || data.fields === void 0 && (missing0 = "fields") || data.requiredPermissions === void 0 && (missing0 = "requiredPermissions") || data.optionalPermissions === void 0 && (missing0 = "optionalPermissions") || data.forbiddenPermissions === void 0 && (missing0 = "forbiddenPermissions")) {
				validate43.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func19.call(schema63.properties, key0)) {
					validate43.errors = [{
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
						validate43.errors = [{
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
							validate43.errors = [{
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
								validate43.errors = [{
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
												validate43.errors = [{
													instancePath: instancePath + "/fields/" + i0,
													schemaPath: "#/$defs/ProviderField/required",
													keyword: "required",
													params: { missingProperty: missing1 },
													message: "must have required property '" + missing1 + "'"
												}];
												return false;
											} else {
												for (const key1 in data4) if (!(key1 === "environment" || key1 === "helpText" || key1 === "id" || key1 === "inputType" || key1 === "label" || key1 === "maxLength" || key1 === "required" || key1 === "secret")) {
													validate43.errors = [{
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
														validate43.errors = [{
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
															validate43.errors = [{
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
																validate43.errors = [{
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
																	validate43.errors = [{
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
																		validate43.errors = [{
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
																			validate43.errors = [{
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
																				validate43.errors = [{
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
																				validate43.errors = [{
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
																					validate43.errors = [{
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
											validate43.errors = [{
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
									validate43.errors = [{
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
											validate43.errors = [{
												instancePath: instancePath + "/forbiddenPermissions/" + i1,
												schemaPath: "#/properties/forbiddenPermissions/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate43.errors = [{
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
											validate43.errors = [{
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
													validate43.errors = [{
														instancePath: instancePath + "/optionalPermissions/" + i2,
														schemaPath: "#/properties/optionalPermissions/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate43.errors = [{
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
													validate43.errors = [{
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
															validate43.errors = [{
																instancePath: instancePath + "/requiredPermissions/" + i3,
																schemaPath: "#/properties/requiredPermissions/items/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
													} else {
														validate43.errors = [{
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
	function validate87(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate87.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.providers === void 0 && (missing0 = "providers")) {
					validate87.errors = [{
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
						validate87.errors = [{
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
										if (!validate43(data0[i0], {
											instancePath: instancePath + "/providers/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate87.errors = [{
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
				validate87.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate87.errors = vErrors;
		return errors === 0;
	}
	validate87.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderDefinition = validate89;
	function validate89(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate89.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.displayName === void 0 && (missing0 = "displayName") || data.environment === void 0 && (missing0 = "environment") || data.available === void 0 && (missing0 = "available") || data.helpText === void 0 && (missing0 = "helpText") || data.fields === void 0 && (missing0 = "fields") || data.requiredPermissions === void 0 && (missing0 = "requiredPermissions") || data.optionalPermissions === void 0 && (missing0 = "optionalPermissions") || data.forbiddenPermissions === void 0 && (missing0 = "forbiddenPermissions")) {
				validate89.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!func19.call(schema63.properties, key0)) {
					validate89.errors = [{
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
						validate89.errors = [{
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
							validate89.errors = [{
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
								validate89.errors = [{
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
												validate89.errors = [{
													instancePath: instancePath + "/fields/" + i0,
													schemaPath: "#/$defs/ProviderField/required",
													keyword: "required",
													params: { missingProperty: missing1 },
													message: "must have required property '" + missing1 + "'"
												}];
												return false;
											} else {
												for (const key1 in data4) if (!(key1 === "environment" || key1 === "helpText" || key1 === "id" || key1 === "inputType" || key1 === "label" || key1 === "maxLength" || key1 === "required" || key1 === "secret")) {
													validate89.errors = [{
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
														validate89.errors = [{
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
															validate89.errors = [{
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
																validate89.errors = [{
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
																	validate89.errors = [{
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
																		validate89.errors = [{
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
																			validate89.errors = [{
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
																				validate89.errors = [{
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
																				validate89.errors = [{
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
																					validate89.errors = [{
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
											validate89.errors = [{
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
									validate89.errors = [{
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
											validate89.errors = [{
												instancePath: instancePath + "/forbiddenPermissions/" + i1,
												schemaPath: "#/properties/forbiddenPermissions/items/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											}];
											return false;
										}
									} else {
										validate89.errors = [{
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
											validate89.errors = [{
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
													validate89.errors = [{
														instancePath: instancePath + "/optionalPermissions/" + i2,
														schemaPath: "#/properties/optionalPermissions/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}];
													return false;
												}
											} else {
												validate89.errors = [{
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
													validate89.errors = [{
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
															validate89.errors = [{
																instancePath: instancePath + "/requiredPermissions/" + i3,
																schemaPath: "#/properties/requiredPermissions/items/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															}];
															return false;
														}
													} else {
														validate89.errors = [{
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
			validate89.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate89.errors = vErrors;
		return true;
	}
	validate89.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.ProviderField = validate90;
	function validate90(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate90.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.label === void 0 && (missing0 = "label") || data.inputType === void 0 && (missing0 = "inputType") || data.required === void 0 && (missing0 = "required") || data.secret === void 0 && (missing0 = "secret") || data.maxLength === void 0 && (missing0 = "maxLength") || data.helpText === void 0 && (missing0 = "helpText") || data.environment === void 0 && (missing0 = "environment")) {
				validate90.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "environment" || key0 === "helpText" || key0 === "id" || key0 === "inputType" || key0 === "label" || key0 === "maxLength" || key0 === "required" || key0 === "secret")) {
					validate90.errors = [{
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
						validate90.errors = [{
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
							validate90.errors = [{
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
								validate90.errors = [{
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
									validate90.errors = [{
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
										validate90.errors = [{
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
											validate90.errors = [{
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
												validate90.errors = [{
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
												validate90.errors = [{
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
													validate90.errors = [{
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
	exports.ProviderSelection = validate91;
	function validate91(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate91.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.providerId === void 0 && (missing0 = "providerId") || data.environment === void 0 && (missing0 = "environment")) {
				validate91.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "environment" || key0 === "providerId")) {
					validate91.errors = [{
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
						validate91.errors = [{
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
							validate91.errors = [{
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
	exports.Remediation = validate92;
	function validate92(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate92.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.label === void 0 && (missing0 = "label")) {
				validate92.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "label")) {
					validate92.errors = [{
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
							validate92.errors = [{
								instancePath: instancePath + "/id",
								schemaPath: "#/properties/id/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate92.errors = [{
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
							validate92.errors = [{
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
	exports.ReplyData = validate93;
	var schema61 = {
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
					"model-gateway"
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
	var schema58 = {
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
					"model-gateway"
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
	function validate36(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate36.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
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
					for (const key0 in data) if (!(key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "projection")) {
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
						if (data.aggregateId !== void 0) {
							let data0 = data.aggregateId;
							const _errs2 = errors;
							if (errors === _errs2) {
								if (typeof data0 === "string") {
									if (func1(data0) < 1) {
										validate36.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate36.errors = [{
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
									validate36.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway")) {
									validate36.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema58.properties.aggregateType.enum },
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
										validate36.errors = [{
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
												validate36.errors = [{
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
												validate36.errors = [{
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
										if (!validate22(data.projection, {
											instancePath: instancePath + "/projection",
											parentData: data,
											parentDataProperty: "projection",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate22.errors : vErrors.concat(validate22.errors);
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
	function validate39(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate39.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate39.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate39.errors = [{
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
									validate39.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate39.errors = [{
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
												validate39.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/id",
													schemaPath: "#/$defs/RuntimeComponent/properties/id/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate39.errors = [{
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
												validate39.errors = [{
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
													validate39.errors = [{
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
								validate39.errors = [{
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
						validate39.errors = [{
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
							validate39.errors = [{
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
								validate39.errors = [{
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
	function validate42(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate42.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.providers === void 0 && (missing0 = "providers")) {
					validate42.errors = [{
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
						validate42.errors = [{
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
										if (!validate43(data0[i0], {
											instancePath: instancePath + "/providers/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate42.errors = [{
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
				validate42.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate42.errors = vErrors;
		return errors === 0;
	}
	validate42.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate47(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate47.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.accounts === void 0 && (missing0 = "accounts")) {
					validate47.errors = [{
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
						validate47.errors = [{
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
										if (!validate25(data0[i0], {
											instancePath: instancePath + "/accounts/" + i0,
											parentData: data0,
											parentDataProperty: i0,
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate25.errors : vErrors.concat(validate25.errors);
											errors = vErrors.length;
										}
										if (!(_errs4 === errors)) break;
									}
								} else {
									validate47.errors = [{
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
				validate47.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate47.errors = vErrors;
		return errors === 0;
	}
	validate47.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate93(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate93.evaluated;
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
														if (data5 > 3 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 3
																},
																message: "must be <= 3"
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
		if (!validate36(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate36.errors : vErrors.concat(validate36.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
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
									if (!(data9 === "workspace" || data9 === "account" || data9 === "model-gateway")) {
										const err21 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/enum",
											keyword: "enum",
											params: { allowedValues: schema61.properties.aggregateType.enum },
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
		if (!validate23(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs35 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs36 = errors;
		if (!validate42(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate42.errors : vErrors.concat(validate42.errors);
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
		if (!validate47(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate47.errors : vErrors.concat(validate47.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs38 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs39 = errors;
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
		var _valid0 = _errs39 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs40 = errors;
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
					const _errs43 = errors;
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
					if (_errs43 === errors) {
						if (data.acknowledged !== void 0) {
							const _errs44 = errors;
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
							var valid6 = _errs44 === errors;
						} else var valid6 = true;
						if (valid6) {
							if (data.detected !== void 0) {
								let data13 = data.detected;
								const _errs46 = errors;
								if (errors === _errs46) {
									if (Array.isArray(data13)) {
										const len0 = data13.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs48 = errors;
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
											if (!(_errs48 === errors)) break;
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
								var valid6 = _errs46 === errors;
							} else var valid6 = true;
							if (valid6) {
								if (data.forbidden !== void 0) {
									let data15 = data.forbidden;
									const _errs50 = errors;
									if (errors === _errs50) {
										if (Array.isArray(data15)) {
											const len1 = data15.length;
											for (let i1 = 0; i1 < len1; i1++) {
												const _errs52 = errors;
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
												if (!(_errs52 === errors)) break;
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
									var valid6 = _errs50 === errors;
								} else var valid6 = true;
								if (valid6) {
									if (data.ipAllowList !== void 0) {
										let data17 = data.ipAllowList;
										const _errs54 = errors;
										if (!Array.isArray(data17) && data17 !== null) {
											const err36 = {
												instancePath: instancePath + "/ipAllowList",
												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
												keyword: "type",
												params: { type: schema49.properties.ipAllowList.type },
												message: "must be array,null"
											};
											if (vErrors === null) vErrors = [err36];
											else vErrors.push(err36);
											errors++;
										}
										if (errors === _errs54) {
											if (Array.isArray(data17)) {
												const len2 = data17.length;
												for (let i2 = 0; i2 < len2; i2++) {
													const _errs56 = errors;
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
													if (!(_errs56 === errors)) break;
												}
											}
										}
										var valid6 = _errs54 === errors;
									} else var valid6 = true;
									if (valid6) {
										if (data.ipAllowListStatus !== void 0) {
											const _errs58 = errors;
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
											var valid6 = _errs58 === errors;
										} else var valid6 = true;
										if (valid6) {
											if (data.scope !== void 0) {
												let data20 = data.scope;
												const _errs60 = errors;
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
														params: { allowedValues: schema49.properties.scope.enum },
														message: "must be equal to one of the allowed values"
													};
													if (vErrors === null) vErrors = [err40];
													else vErrors.push(err40);
													errors++;
												}
												var valid6 = _errs60 === errors;
											} else var valid6 = true;
											if (valid6) {
												if (data.unsupported !== void 0) {
													let data21 = data.unsupported;
													const _errs62 = errors;
													if (errors === _errs62) {
														if (Array.isArray(data21)) {
															const len3 = data21.length;
															for (let i3 = 0; i3 < len3; i3++) {
																const _errs64 = errors;
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
																if (!(_errs64 === errors)) break;
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
		var _valid0 = _errs40 === errors;
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
			validate93.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate93.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate93.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	exports.ResultEnvelope = validate101;
	function validate35(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate35.evaluated;
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
														if (data5 > 3 || isNaN(data5)) {
															const err8 = {
																instancePath: instancePath + "/storageSchemaVersion",
																schemaPath: "#/$defs/Workspace/properties/storageSchemaVersion/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 3
																},
																message: "must be <= 3"
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
		if (!validate36(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate36.errors : vErrors.concat(validate36.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs19 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs20 = errors;
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
									if (!(data9 === "workspace" || data9 === "account" || data9 === "model-gateway")) {
										const err21 = {
											instancePath: instancePath + "/aggregateType",
											schemaPath: "#/$defs/SubscriptionAck/properties/aggregateType/enum",
											keyword: "enum",
											params: { allowedValues: schema61.properties.aggregateType.enum },
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
		if (!validate23(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs35 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs36 = errors;
		if (!validate42(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate42.errors : vErrors.concat(validate42.errors);
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
		if (!validate47(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate47.errors : vErrors.concat(validate47.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs38 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs39 = errors;
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
		var _valid0 = _errs39 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) {
			if (props0 !== true) props0 = true;
		}
		const _errs40 = errors;
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
					const _errs43 = errors;
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
					if (_errs43 === errors) {
						if (data.acknowledged !== void 0) {
							const _errs44 = errors;
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
							var valid6 = _errs44 === errors;
						} else var valid6 = true;
						if (valid6) {
							if (data.detected !== void 0) {
								let data13 = data.detected;
								const _errs46 = errors;
								if (errors === _errs46) {
									if (Array.isArray(data13)) {
										const len0 = data13.length;
										for (let i0 = 0; i0 < len0; i0++) {
											const _errs48 = errors;
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
											if (!(_errs48 === errors)) break;
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
								var valid6 = _errs46 === errors;
							} else var valid6 = true;
							if (valid6) {
								if (data.forbidden !== void 0) {
									let data15 = data.forbidden;
									const _errs50 = errors;
									if (errors === _errs50) {
										if (Array.isArray(data15)) {
											const len1 = data15.length;
											for (let i1 = 0; i1 < len1; i1++) {
												const _errs52 = errors;
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
												if (!(_errs52 === errors)) break;
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
									var valid6 = _errs50 === errors;
								} else var valid6 = true;
								if (valid6) {
									if (data.ipAllowList !== void 0) {
										let data17 = data.ipAllowList;
										const _errs54 = errors;
										if (!Array.isArray(data17) && data17 !== null) {
											const err36 = {
												instancePath: instancePath + "/ipAllowList",
												schemaPath: "#/$defs/PermissionReview/properties/ipAllowList/type",
												keyword: "type",
												params: { type: schema49.properties.ipAllowList.type },
												message: "must be array,null"
											};
											if (vErrors === null) vErrors = [err36];
											else vErrors.push(err36);
											errors++;
										}
										if (errors === _errs54) {
											if (Array.isArray(data17)) {
												const len2 = data17.length;
												for (let i2 = 0; i2 < len2; i2++) {
													const _errs56 = errors;
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
													if (!(_errs56 === errors)) break;
												}
											}
										}
										var valid6 = _errs54 === errors;
									} else var valid6 = true;
									if (valid6) {
										if (data.ipAllowListStatus !== void 0) {
											const _errs58 = errors;
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
											var valid6 = _errs58 === errors;
										} else var valid6 = true;
										if (valid6) {
											if (data.scope !== void 0) {
												let data20 = data.scope;
												const _errs60 = errors;
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
														params: { allowedValues: schema49.properties.scope.enum },
														message: "must be equal to one of the allowed values"
													};
													if (vErrors === null) vErrors = [err40];
													else vErrors.push(err40);
													errors++;
												}
												var valid6 = _errs60 === errors;
											} else var valid6 = true;
											if (valid6) {
												if (data.unsupported !== void 0) {
													let data21 = data.unsupported;
													const _errs62 = errors;
													if (errors === _errs62) {
														if (Array.isArray(data21)) {
															const len3 = data21.length;
															for (let i3 = 0; i3 < len3; i3++) {
																const _errs64 = errors;
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
																if (!(_errs64 === errors)) break;
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
		var _valid0 = _errs40 === errors;
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
			validate35.errors = vErrors;
			return false;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate35.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate35.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate34(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate34.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate34.errors = [{
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
						validate34.errors = [{
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
							if (!validate35(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate35.errors : vErrors.concat(validate35.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate34.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate34.errors = [{
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
												validate34.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate34.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate34.errors = [{
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
											validate34.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate34.errors = [{
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
													validate34.errors = [{
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
														validate34.errors = [{
															instancePath: instancePath + "/stateVersion",
															schemaPath: "#/properties/stateVersion/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate34.errors = [{
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
				validate34.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate34.errors = vErrors;
		return errors === 0;
	}
	validate34.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate53(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate53.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.error === void 0 && (missing0 = "error")) {
					validate53.errors = [{
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
						validate53.errors = [{
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
							if (!validate54(data.error, {
								instancePath: instancePath + "/error",
								parentData: data,
								parentDataProperty: "error",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate54.errors : vErrors.concat(validate54.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate53.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (false !== data1) {
									validate53.errors = [{
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
												validate53.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate53.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate53.errors = [{
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
											validate53.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate53.errors = [{
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
													validate53.errors = [{
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
				validate53.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate53.errors = vErrors;
		return errors === 0;
	}
	validate53.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	function validate101(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate101.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		const _errs1 = errors;
		if (!validate34(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate34.errors : vErrors.concat(validate34.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		valid0 = valid0 || _valid0;
		if (_valid0) var props0 = true;
		const _errs2 = errors;
		if (!validate53(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData,
			dynamicAnchors
		})) {
			vErrors = vErrors === null ? validate53.errors : vErrors.concat(validate53.errors);
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
	exports.RuntimeComponent = validate104;
	function validate104(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate104.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.id === void 0 && (missing0 = "id") || data.status === void 0 && (missing0 = "status") || data.message === void 0 && (missing0 = "message")) {
				validate104.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "id" || key0 === "message" || key0 === "status")) {
					validate104.errors = [{
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
							validate104.errors = [{
								instancePath: instancePath + "/id",
								schemaPath: "#/properties/id/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate104.errors = [{
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
							validate104.errors = [{
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
								validate104.errors = [{
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
			validate104.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate104.errors = vErrors;
		return true;
	}
	validate104.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.RuntimeStatus = validate105;
	function validate105(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate105.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.components === void 0 && (missing0 = "components") || data.modelAvailable === void 0 && (missing0 = "modelAvailable") || data.liveExecutionAvailable === void 0 && (missing0 = "liveExecutionAvailable")) {
				validate105.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "components" || key0 === "liveExecutionAvailable" || key0 === "modelAvailable")) {
					validate105.errors = [{
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
									validate105.errors = [{
										instancePath: instancePath + "/components/" + i0,
										schemaPath: "#/$defs/RuntimeComponent/required",
										keyword: "required",
										params: { missingProperty: missing1 },
										message: "must have required property '" + missing1 + "'"
									}];
									return false;
								} else {
									for (const key1 in data1) if (!(key1 === "id" || key1 === "message" || key1 === "status")) {
										validate105.errors = [{
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
												validate105.errors = [{
													instancePath: instancePath + "/components/" + i0 + "/id",
													schemaPath: "#/$defs/RuntimeComponent/properties/id/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate105.errors = [{
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
												validate105.errors = [{
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
													validate105.errors = [{
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
								validate105.errors = [{
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
						validate105.errors = [{
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
							validate105.errors = [{
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
								validate105.errors = [{
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
			validate105.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate105.errors = vErrors;
		return true;
	}
	validate105.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Snapshot = validate106;
	function validate106(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate106.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.projection === void 0 && (missing0 = "projection") || data.lastSequence === void 0 && (missing0 = "lastSequence")) {
					validate106.errors = [{
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
						validate106.errors = [{
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
										validate106.errors = [{
											instancePath: instancePath + "/aggregateId",
											schemaPath: "#/properties/aggregateId/minLength",
											keyword: "minLength",
											params: { limit: 1 },
											message: "must NOT have fewer than 1 characters"
										}];
										return false;
									}
								} else {
									validate106.errors = [{
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
									validate106.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}];
									return false;
								}
								if (!(data1 === "workspace" || data1 === "account" || data1 === "model-gateway")) {
									validate106.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/enum",
										keyword: "enum",
										params: { allowedValues: schema58.properties.aggregateType.enum },
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
										validate106.errors = [{
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
												validate106.errors = [{
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
												validate106.errors = [{
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
										if (!validate22(data.projection, {
											instancePath: instancePath + "/projection",
											parentData: data,
											parentDataProperty: "projection",
											rootData,
											dynamicAnchors
										})) {
											vErrors = vErrors === null ? validate22.errors : vErrors.concat(validate22.errors);
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
				validate106.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate106.errors = vErrors;
		return errors === 0;
	}
	validate106.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Subscribe = validate108;
	function validate108(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate108.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence")) {
				validate108.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType")) {
					validate108.errors = [{
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
						validate108.errors = [{
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
							validate108.errors = [{
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
							validate108.errors = [{
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
								validate108.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/maxLength",
									keyword: "maxLength",
									params: { limit: 128 },
									message: "must NOT have more than 128 characters"
								}];
								return false;
							} else if (func1(data1) < 1) {
								validate108.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate108.errors = [{
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
									validate108.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/maxLength",
										keyword: "maxLength",
										params: { limit: 64 },
										message: "must NOT have more than 64 characters"
									}];
									return false;
								} else if (func1(data2) < 1) {
									validate108.errors = [{
										instancePath: instancePath + "/aggregateType",
										schemaPath: "#/properties/aggregateType/minLength",
										keyword: "minLength",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 characters"
									}];
									return false;
								}
							} else {
								validate108.errors = [{
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
			validate108.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate108.errors = vErrors;
		return true;
	}
	validate108.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SubscriptionAck = validate109;
	function validate109(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate109.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.aggregateType === void 0 && (missing0 = "aggregateType") || data.aggregateId === void 0 && (missing0 = "aggregateId") || data.afterSequence === void 0 && (missing0 = "afterSequence") || data.lastSequence === void 0 && (missing0 = "lastSequence") || data.replayedCount === void 0 && (missing0 = "replayedCount")) {
				validate109.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "afterSequence" || key0 === "aggregateId" || key0 === "aggregateType" || key0 === "lastSequence" || key0 === "replayedCount")) {
					validate109.errors = [{
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
						validate109.errors = [{
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
							validate109.errors = [{
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
							validate109.errors = [{
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
								validate109.errors = [{
									instancePath: instancePath + "/aggregateId",
									schemaPath: "#/properties/aggregateId/minLength",
									keyword: "minLength",
									params: { limit: 1 },
									message: "must NOT have fewer than 1 characters"
								}];
								return false;
							}
						} else {
							validate109.errors = [{
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
								validate109.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								}];
								return false;
							}
							if (!(data2 === "workspace" || data2 === "account" || data2 === "model-gateway")) {
								validate109.errors = [{
									instancePath: instancePath + "/aggregateType",
									schemaPath: "#/properties/aggregateType/enum",
									keyword: "enum",
									params: { allowedValues: schema61.properties.aggregateType.enum },
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
									validate109.errors = [{
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
										validate109.errors = [{
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
										validate109.errors = [{
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
										validate109.errors = [{
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
											validate109.errors = [{
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
											validate109.errors = [{
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
			validate109.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate109.errors = vErrors;
		return true;
	}
	validate109.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.SuccessEnvelope = validate110;
	function validate110(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate110.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (errors === 0) {
			if (data && typeof data == "object" && !Array.isArray(data)) {
				let missing0;
				if (data.requestId === void 0 && (missing0 = "requestId") || data.schemaVersion === void 0 && (missing0 = "schemaVersion") || data.ok === void 0 && (missing0 = "ok") || data.data === void 0 && (missing0 = "data")) {
					validate110.errors = [{
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
						validate110.errors = [{
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
							if (!validate35(data.data, {
								instancePath: instancePath + "/data",
								parentData: data,
								parentDataProperty: "data",
								rootData,
								dynamicAnchors
							})) {
								vErrors = vErrors === null ? validate35.errors : vErrors.concat(validate35.errors);
								errors = vErrors.length;
							}
							var valid0 = _errs2 === errors;
						} else var valid0 = true;
						if (valid0) {
							if (data.ok !== void 0) {
								let data1 = data.ok;
								const _errs3 = errors;
								if (typeof data1 !== "boolean") {
									validate110.errors = [{
										instancePath: instancePath + "/ok",
										schemaPath: "#/properties/ok/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}];
									return false;
								}
								if (true !== data1) {
									validate110.errors = [{
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
												validate110.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/maxLength",
													keyword: "maxLength",
													params: { limit: 128 },
													message: "must NOT have more than 128 characters"
												}];
												return false;
											} else if (func1(data2) < 1) {
												validate110.errors = [{
													instancePath: instancePath + "/requestId",
													schemaPath: "#/properties/requestId/minLength",
													keyword: "minLength",
													params: { limit: 1 },
													message: "must NOT have fewer than 1 characters"
												}];
												return false;
											}
										} else {
											validate110.errors = [{
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
											validate110.errors = [{
												instancePath: instancePath + "/schemaVersion",
												schemaPath: "#/properties/schemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (1 !== data3) {
											validate110.errors = [{
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
													validate110.errors = [{
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
														validate110.errors = [{
															instancePath: instancePath + "/stateVersion",
															schemaPath: "#/properties/stateVersion/minLength",
															keyword: "minLength",
															params: { limit: 1 },
															message: "must NOT have fewer than 1 characters"
														}];
														return false;
													}
												} else {
													validate110.errors = [{
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
				validate110.errors = [{
					instancePath,
					schemaPath: "#/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				}];
				return false;
			}
		}
		validate110.errors = vErrors;
		return errors === 0;
	}
	validate110.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.TradeXError = validate112;
	function validate112(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate112.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.category === void 0 && (missing0 = "category") || data.code === void 0 && (missing0 = "code") || data.message === void 0 && (missing0 = "message") || data.retryable === void 0 && (missing0 = "retryable") || data.blocking === void 0 && (missing0 = "blocking") || data.remediationActions === void 0 && (missing0 = "remediationActions")) {
				validate112.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "blocking" || key0 === "category" || key0 === "code" || key0 === "message" || key0 === "remediationActions" || key0 === "retryable")) {
					validate112.errors = [{
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
						validate112.errors = [{
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
							validate112.errors = [{
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
								validate112.errors = [{
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
									validate112.errors = [{
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
													validate112.errors = [{
														instancePath: instancePath + "/remediationActions/" + i0,
														schemaPath: "#/$defs/Remediation/required",
														keyword: "required",
														params: { missingProperty: missing1 },
														message: "must have required property '" + missing1 + "'"
													}];
													return false;
												} else {
													for (const key1 in data5) if (!(key1 === "id" || key1 === "label")) {
														validate112.errors = [{
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
																validate112.errors = [{
																	instancePath: instancePath + "/remediationActions/" + i0 + "/id",
																	schemaPath: "#/$defs/Remediation/properties/id/minLength",
																	keyword: "minLength",
																	params: { limit: 1 },
																	message: "must NOT have fewer than 1 characters"
																}];
																return false;
															}
														} else {
															validate112.errors = [{
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
																validate112.errors = [{
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
												validate112.errors = [{
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
										validate112.errors = [{
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
											validate112.errors = [{
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
			validate112.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate112.errors = vErrors;
		return true;
	}
	validate112.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
	exports.Workspace = validate113;
	function validate113(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate113.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId") || data.name === void 0 && (missing0 = "name") || data.baseCurrency === void 0 && (missing0 = "baseCurrency") || data.path === void 0 && (missing0 = "path") || data.createdAt === void 0 && (missing0 = "createdAt") || data.lastOpenedAt === void 0 && (missing0 = "lastOpenedAt") || data.storageSchemaVersion === void 0 && (missing0 = "storageSchemaVersion")) {
				validate113.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "baseCurrency" || key0 === "createdAt" || key0 === "lastOpenedAt" || key0 === "name" || key0 === "path" || key0 === "storageSchemaVersion" || key0 === "workspaceId")) {
					validate113.errors = [{
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
						validate113.errors = [{
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
							validate113.errors = [{
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
								validate113.errors = [{
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
									validate113.errors = [{
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
										validate113.errors = [{
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
											validate113.errors = [{
												instancePath: instancePath + "/storageSchemaVersion",
												schemaPath: "#/properties/storageSchemaVersion/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											}];
											return false;
										}
										if (typeof data5 == "number") {
											if (data5 > 3 || isNaN(data5)) {
												validate113.errors = [{
													instancePath: instancePath + "/storageSchemaVersion",
													schemaPath: "#/properties/storageSchemaVersion/maximum",
													keyword: "maximum",
													params: {
														comparison: "<=",
														limit: 3
													},
													message: "must be <= 3"
												}];
												return false;
											} else if (data5 < 1 || isNaN(data5)) {
												validate113.errors = [{
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
													validate113.errors = [{
														instancePath: instancePath + "/workspaceId",
														schemaPath: "#/properties/workspaceId/minLength",
														keyword: "minLength",
														params: { limit: 1 },
														message: "must NOT have fewer than 1 characters"
													}];
													return false;
												}
											} else {
												validate113.errors = [{
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
	exports.WorkspaceQuery = validate114;
	function validate114(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		const evaluated0 = validate114.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			let missing0;
			if (data.workspaceId === void 0 && (missing0 = "workspaceId")) {
				validate114.errors = [{
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: missing0 },
					message: "must have required property '" + missing0 + "'"
				}];
				return false;
			} else {
				for (const key0 in data) if (!(key0 === "workspaceId")) {
					validate114.errors = [{
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
							validate114.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/maxLength",
								keyword: "maxLength",
								params: { limit: 128 },
								message: "must NOT have more than 128 characters"
							}];
							return false;
						} else if (func1(data0) < 1) {
							validate114.errors = [{
								instancePath: instancePath + "/workspaceId",
								schemaPath: "#/properties/workspaceId/minLength",
								keyword: "minLength",
								params: { limit: 1 },
								message: "must NOT have fewer than 1 characters"
							}];
							return false;
						}
					} else {
						validate114.errors = [{
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
			validate114.errors = [{
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			}];
			return false;
		}
		validate114.errors = vErrors;
		return true;
	}
	validate114.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
}));
//#endregion
export default require_ipc_validators_input();
