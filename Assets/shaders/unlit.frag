in vec3 vNormal;
in vec2 vUvChannels[maxUvChannelCount];

out vec4 fragColor;

vec4 getAlbedoColor(vec2 texCoordsOffset) {
	if (useAlbedoTexture == 1) {
		return me_texture_2d(
			albedoTexture,
			vUvChannels[albedoTextureUvChannelId] + texCoordsOffset
		);
	} else {
		return vec4(1.0f);
	}
}

void main()
{
	vec3 albedo = vec3(1.0, 1.0, 1.0);

	vec2 texCoordsOffset = vec2(0.0f, 0.0f);

	vec4 tmp = getAlbedoColor(texCoordsOffset);
	albedo = vec3(tmp);
	float alpha = tmp.a;

	fragColor = vec4(max(albedo, emissiveColor) * albedoColor, alpha);

	if (fragColor.a < 0.1)
	{
		me_discard();
	}
}
