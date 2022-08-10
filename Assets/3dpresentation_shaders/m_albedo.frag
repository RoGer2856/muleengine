#version 130

uniform vec4 albedoColor;

uniform vec3 eyePosition;

uniform float albedoTextureFactor;
uniform sampler2D albedoTexture;
uniform float normalTextureFactor;
uniform sampler2D normalTexture;
uniform float displacementTextureFactor;
uniform sampler2D displacementTexture;
uniform float specularTextureFactor;
uniform sampler2D specularTexture;

uniform mat4 viewMatrix;

in vec3 vNormal;
in vec2 vTexCoords;
in vec4 vWorldPos;

float pointLightDiffuse(vec3 lp)
{
	return 1.0f;
}

float pointLightSpecular(vec3 lp)
{
	return 1.0f;
}

float directionalLightDiffuse(vec3 ld)
{
	return clamp(dot(normalize(vNormal), -normalize(ld)), 0.0f, 1.0f);
}

float directionalLightSpecular(vec3 ld, float shininess)
{
	if (shininess == 0.0f)
	{
		return 0.0f;
	}
	else
	{
		vec3 h = normalize(vWorldPos.xyz - eyePosition) + normalize(ld);
		return pow(clamp(dot(normalize(vNormal), -normalize(h)), 0.0, 1.0), shininess);
	}
}

void main()
{
	vec4 ambient = vec4(0.1f, 0.1f, 0.1f, 0.0f);

	vec4 albedo = albedoColor;
	albedo = albedo * (1.0f - albedoTextureFactor) + texture(albedoTexture, vTexCoords) * albedoTextureFactor;

	float alpha = albedo.a;

	float shininess = texture(specularTexture, vTexCoords).r * specularTextureFactor * 128.0f;

	vec4 lightDiffuseIntensity = vec4(0.1f, 0.1f, 0.1f, 0.1f);
	vec4 lightSpecularIntensity = vec4(0.0f, 0.0f, 0.0f, 0.0f);

	lightDiffuseIntensity += vec4(1.0f, 1.0f, 1.0f, 1.0f) * directionalLightDiffuse(vec3(1.0f, -1.0f, -1.0f));
	lightSpecularIntensity += vec4(1.0f, 1.0f, 1.0f, 1.0f) * directionalLightSpecular(vec3(1.0f, -1.0f, -1.0f), shininess);

	lightDiffuseIntensity += vec4(0.1f, 0.1f, 0.4f, 1.0f) * directionalLightDiffuse(vec3(-1.0f, -1.0f, -1.0f));
	lightSpecularIntensity += vec4(0.1f, 0.1f, 0.4f, 1.0f) * directionalLightSpecular(vec3(-1.0f, -1.0f, -1.0f), shininess);

	vec4 resultColor;

	// albedo
	resultColor = albedo;

	// ambient
//	resultColor = ambient * albedoColor;

	// ambient + diffuse
//	resultColor = albedo * (ambient + lightDiffuseIntensity);

	// ambient + diffuse + specular
//	resultColor = max(albedo * (ambient + lightDiffuseIntensity), lightSpecularIntensity);

	// ambient + diffuse + specular + normal map

	// ambient + diffuse + specular + normal map + displacement map

	resultColor.a = alpha;
	gl_FragColor = resultColor;
}
