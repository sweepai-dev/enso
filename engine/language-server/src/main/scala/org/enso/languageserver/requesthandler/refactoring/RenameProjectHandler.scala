package org.enso.languageserver.requesthandler.refactoring

import java.util.UUID

import akka.actor.{Actor, ActorRef, Cancellable, Props}
import com.typesafe.scalalogging.LazyLogging
import org.enso.jsonrpc._
import org.enso.languageserver.refactoring.RefactoringApi.RenameProject
import org.enso.languageserver.requesthandler.RequestTimeout
import org.enso.languageserver.util.UnhandledLogging
import org.enso.polyglot.runtime.Runtime.Api

import scala.concurrent.duration.FiniteDuration

/** A request handler for `refactoring/renameProject` commands.
  *
  * @param timeout a request timeout
  * @param runtimeConnector a reference to the runtime connector
  */
class RenameProjectHandler(timeout: FiniteDuration, runtimeConnector: ActorRef)
    extends Actor
    with LazyLogging
    with UnhandledLogging {

  import context.dispatcher

  override def receive: Receive = requestStage

  private def requestStage: Receive = {
    case Request(RenameProject, id, params: RenameProject.Params) =>
      val payload =
        Api.RenameProject(params.namespace, params.oldName, params.newName)
      runtimeConnector ! Api.Request(UUID.randomUUID(), payload)
      val cancellable =
        context.system.scheduler.scheduleOnce(timeout, self, RequestTimeout)
      context.become(
        responseStage(
          id,
          sender(),
          cancellable
        )
      )
  }

  private def responseStage(
    id: Id,
    replyTo: ActorRef,
    cancellable: Cancellable
  ): Receive = {
    case RequestTimeout =>
      logger.error("Request [{}] timed out.", id)
      replyTo ! ResponseError(Some(id), Errors.RequestTimeout)
      context.stop(self)

    case Api.Response(_, Api.ProjectRenamed(_, _)) =>
      replyTo ! ResponseResult(RenameProject, id, Unused)
      cancellable.cancel()
      context.stop(self)
  }

}

object RenameProjectHandler {

  /** Creates configuration object used to create a [[RenameProjectHandler]].
    *
    * @param timeout request timeout
    * @param runtimeConnector reference to the runtime connector
    */
  def props(timeout: FiniteDuration, runtimeConnector: ActorRef): Props =
    Props(new RenameProjectHandler(timeout, runtimeConnector))

}
