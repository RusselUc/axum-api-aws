mod domains;

// use domains::auth::dynamo::CognitoUserManager;
use domains::auth::dynamo::CognitoUserManager;
use dotenv::dotenv;
use std::env;
use std::io::{self, Write};

fn read_email() -> Result<String, Box<dyn std::error::Error>> {
    // Verificar variable de entorno primero
    if let Ok(email) = std::env::var("USER_EMAIL") {
        println!("📧 Usando email desde variable de entorno: {}", email);
        return Ok(email);
    }

    // Modo interactivo
    print!("📧 Ingrese su dirección de email: ");
    std::io::stdout().flush()?;
    let mut email = String::new();
    std::io::stdin().read_line(&mut email)?;

    let email = email.trim().to_string();

    // Validación básica
    if !email.contains('@') || !email.contains('.') {
        return Err("Email inválido. Debe contener @ y un dominio válido.".into());
    }

    println!("✅ Email: {}", email);
    Ok(email)
}

fn read_password() -> Result<String, Box<dyn std::error::Error>> {
    // Verificar variable de entorno primero
    if let Ok(password) = std::env::var("USER_PASSWORD") {
        println!("🔑 Usando contraseña desde variable de entorno");
        return Ok(password);
    }

    // Modo interactivo
    print!("🔑 Ingrese su contraseña (mín. 8 caracteres): ");
    std::io::stdout().flush()?;
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;

    let password = password.trim().to_string();

    if password.len() < 8 {
        return Err("La contraseña debe tener al menos 8 caracteres.".into());
    }

    Ok(password)
}

fn read_confirmation_code() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(code) = std::env::var("CONFIRMATION_CODE") {
        println!("📝 Usando código desde variable de entorno: {}", code);
        return Ok(code);
    }

    // Modo interactivo
    println!("📧 Revisa tu email para obtener el código de confirmación.");
    print!("📝 Ingrese el código de confirmación (6 dígitos): ");
    std::io::stdout().flush()?;
    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;

    let code = code.trim().to_string();

    // Validación del código
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        println!(
            "⚠️  Advertencia: El código debería ser de 6 dígitos, pero intentaremos con: {}",
            code
        );
    }

    Ok(code)
}

fn confirm_retry(message: &str) -> bool {
    print!("{} (s/n): ", message);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    matches!(
        input.trim().to_lowercase().as_str(),
        "s" | "si" | "y" | "yes"
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    run_dynamo().await
}

pub async fn run_dynamo() -> Result<(), Box<dyn std::error::Error>> {
    let manager = CognitoUserManager::new().await?;
    manager.ensure_users_table().await?;
    let users = manager.list_users().await?;
    println!("Usuarios en Cognito:");
    for user in users {
        println!(" - {}", user);
    }

    println!("\n🚀 === COGNITO + DYNAMODB - REGISTRO Y AUTENTICACIÓN ===");

    // === PRIMER USUARIO - FLUJO CLIENTE ===
    println!("\n📝 === REGISTRO DE USUARIO (FLUJO CLIENTE) ===");

    let email1 = read_email()?;
    let password1 = read_password()?;

    println!("🔄 Registrando usuario en Cognito...");
    let username1 = manager
        .register_user_client_flow(&email1, &password1)
        .await?;

    println!("✅ Usuario registrado con username generado: {}", username1);
    println!("📬 Se ha enviado un código de confirmación a: {}", email1);

    // Confirmación con reintentos
    loop {
        match read_confirmation_code() {
            Ok(confirmation_code) => {
                println!("📝 Código recibido: {}", confirmation_code);
                println!("🔄 Confirmando usuario...");

                match manager.confirm_user(&username1, &confirmation_code).await {
                    Ok(_) => {
                        manager
                            .save_user_to_dynamo(&username1, email1.as_str())
                            .await?;
                        println!("✅ Usuario confirmado exitosamente");
                        break;
                    }
                    Err(e) => {
                        eprintln!("❌ Error confirmando usuario: {:?}", e);
                        eprintln!("💡 Posibles causas:");
                        eprintln!("   - Código incorrecto");
                        eprintln!("   - Código expirado (válido por 24 horas)");
                        eprintln!("   - Código ya usado");

                        if confirm_retry("¿Deseas intentar con otro código?") {
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error leyendo código: {}", e);
                if confirm_retry("¿Deseas intentar nuevamente?") {
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Autenticación del primer usuario
    println!("\n🔐 === AUTENTICACIÓN (FLUJO CLIENTE) ===");
    println!("🔄 Autenticando usuario...");
    match manager
        .authenticate_user_client_flow(&username1, &password1)
        .await
    {
        Ok(Some(token)) => {
            println!(
                "✅ Token obtenido (flujo cliente): {}...",
                &token[..50.min(token.len())]
            );
        }
        Ok(None) => {
            println!("❌ No se pudo obtener token en flujo cliente");
        }
        Err(e) => {
            eprintln!("❌ Error en autenticación: {:?}", e);
        }
    }

    // === SEGUNDO USUARIO - MÉTODOS ADMIN ===
    if confirm_retry("\n¿Deseas registrar un segundo usuario usando métodos admin?") {
        println!("\n👨‍💼 === MÉTODOS ADMIN ===");

        let email2 = read_email()?;
        let password2 = read_password()?;

        println!("🔄 Registrando usuario admin...");
        let admin_username = manager.register_user(&email2, &password2).await?;

        println!(
            "✅ Usuario admin registrado con username: {}",
            admin_username
        );
        println!("📬 Se ha enviado un código de confirmación a: {}", email2);

        // Confirmación del usuario admin
        loop {
            match read_confirmation_code() {
                Ok(admin_confirmation_code) => {
                    println!("📝 Código admin recibido: {}", admin_confirmation_code);
                    println!("🔄 Confirmando usuario admin...");

                    match manager
                        .confirm_user(&admin_username, &admin_confirmation_code)
                        .await
                    {
                        Ok(_) => {
                            println!("✅ Usuario admin confirmado exitosamente");
                            break;
                        }
                        Err(e) => {
                            eprintln!("❌ Error confirmando usuario admin: {:?}", e);
                            if confirm_retry("¿Deseas intentar con otro código?") {
                                continue;
                            } else {
                                return Err(e.into());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error leyendo código admin: {}", e);
                    if confirm_retry("¿Deseas intentar nuevamente?") {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Autenticación del usuario admin
        println!("🔄 Autenticando usuario admin...");
        match manager.authenticate_user(&admin_username, &password2).await {
            Ok(Some(token)) => {
                println!(
                    "✅ Token obtenido (admin): {}...",
                    &token[..50.min(token.len())]
                );

                if let Some(user) = manager.get_user_from_token(&token).await? {
                    println!("👤 Usuario autenticado: {:?}", user);
                }
            }
            Ok(None) => {
                println!("❌ No se pudo obtener token en admin");
            }
            Err(e) => {
                eprintln!("❌ Error en autenticación admin: {:?}", e);
            }
        }
    }

    println!("\n🎉 === PROCESO COMPLETADO ===");
    println!("✅ Cognito: Usuarios registrados y autenticados");
    println!("✅ DynamoDB: Tabla verificada y lista");

    Ok(())
}
