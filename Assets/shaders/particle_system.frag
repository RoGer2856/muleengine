in vec2 vTexCoords;

out vec4 fragColor;

void main()
{
	vec4 albedo = vec4(1.0, 1.0, 1.0, 1.0);
	vec4 ambient = vec4(0.1, 0.1, 0.1, 0.0);

	albedo = albedo * (1.0 - textureBlend) + me_texture_2d(albedoTexture, vTexCoords) * textureBlend;

	float alpha = albedo.a;

	vec4 resultColor = albedo * albedoColor;
	resultColor.a = alpha;
	fragColor = resultColor;

	if (fragColor.a < 0.1)
	{
		me_discard();
	}
}
